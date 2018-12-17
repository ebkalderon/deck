pub use self::error::{CreationError, OpenError};

use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::fs;
use std::io::{Error as IoError, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use diesel::sqlite::SqliteConnection;
use diesel::Connection;
use filetime::FileTime;
use futures::{future, Future, IntoFuture, Stream};
use hyper::{Client, Uri};
use ignore::{WalkBuilder, WalkState};
use indicatif::{MultiProgress, ProgressBar};
use tokio::{self, fs::File};

use manifest::Manifest;

mod error;

const MANIFESTS_SUBDIR: &str = "manifests";
const OUTPUTS_SUBDIR: &str = "outputs";
const SOURCES_SUBDIR: &str = "sources";
const TEMP_SUBDIR: &str = "tmp";
const VAR_SUBDIR: &str = "var";
const DB_FILE_NAME: &str = "index.db";

embed_migrations!("migrations");

pub struct StoreDirectory {
    prefix: PathBuf,
    index: SqliteConnection,
    progress: MultiProgress,
}

impl StoreDirectory {
    pub fn open(path: PathBuf) -> Result<Self, OpenError> {
        let prefix = fs::read_dir(&path)
            .map_err(OpenError::InvalidPath)
            .and_then(|_| fs::canonicalize(path).map_err(OpenError::InvalidPath))?;

        let conn_str = prefix.join(VAR_SUBDIR).join(DB_FILE_NAME);
        let database = fs::metadata(&conn_str).map_err(OpenError::MissingIndex)?;

        if !database.file_type().is_file() {
            return Err(OpenError::InvalidIndex);
        }

        Ok(StoreDirectory {
            prefix,
            index: SqliteConnection::establish(&conn_str.to_string_lossy())?,
            progress: MultiProgress::new(),
        })
    }

    pub fn create_in(path: PathBuf) -> Result<Self, CreationError> {
        match fs::read_dir(&path) {
            Ok(_) => return Err(CreationError::AlreadyExists(path.into())),
            Err(err) => {
                if err.kind() != ErrorKind::NotFound {
                    return Err(CreationError::AccessDenied(err));
                }
            }
        }

        let tmp_dir: PathBuf = format!("{}.tmp", path.to_string_lossy()).into();
        let index = new_store_transaction(&tmp_dir, |dir| {
            fs::create_dir(&dir).map_err(CreationError::CreateDirectory)?;
            fs::create_dir(&dir.join(MANIFESTS_SUBDIR)).map_err(CreationError::CreateDirectory)?;
            fs::create_dir(&dir.join(OUTPUTS_SUBDIR)).map_err(CreationError::CreateDirectory)?;
            fs::create_dir(&dir.join(SOURCES_SUBDIR)).map_err(CreationError::CreateDirectory)?;
            fs::create_dir(&dir.join(TEMP_SUBDIR)).map_err(CreationError::CreateDirectory)?;
            fs::create_dir(&dir.join(VAR_SUBDIR)).map_err(CreationError::CreateDirectory)?;

            let conn_str = dir.join(VAR_SUBDIR).join(DB_FILE_NAME);
            let conn = SqliteConnection::establish(&conn_str.to_string_lossy())?;
            embedded_migrations::run(&conn)?;

            Ok(conn)
        })?;

        fs::rename(tmp_dir, &path).map_err(CreationError::Rename)?;

        Ok(StoreDirectory {
            prefix: fs::canonicalize(path).map_err(CreationError::AccessDenied)?,
            index,
            progress: MultiProgress::new(),
        })
    }

    pub fn add_manifest(&self, _manifest: Manifest) -> Result<Manifest, IoError> {
        unimplemented!()
    }

    pub fn add_source(&self, _path: &Path) -> Result<PathBuf, IoError> {
        unimplemented!()
    }

    pub fn download_manifest(&self, _uri: Uri, _hash: String) -> Result<Manifest, IoError> {
        unimplemented!()
    }

    pub fn download_source(&self, uri: Uri) -> Result<Download, ()> {
        use hyper::header::CONTENT_LENGTH;

        let name = Path::new(uri.path()).file_name().unwrap().to_os_string();

        let prefix = self.prefix.clone();
        let tmp = prefix.join(TEMP_SUBDIR).join(name);
        let progress = ProgressBar::new(0);
        progress.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes:>7}/{total_bytes:7} {msg}")
                .progress_chars("##-"),
        );

        let progress = Arc::new(self.progress.add(progress));
        let prog = progress.clone();
        let future = Box::new(
            File::create(tmp.clone())
                .map_err(|e| eprintln!("failed to create file: {}", e))
                .and_then(move |file| {
                    Client::new()
                        .get(uri.clone())
                        .map_err(|e| eprintln!("failed to connect to URI: {}", e))
                        .and_then(move |resp| {
                            let len = resp
                                .headers()
                                .get(CONTENT_LENGTH)
                                .and_then(|len| len.to_str().ok())
                                .and_then(|len| len.parse::<u64>().ok());

                            if let Some(len) = len {
                                prog.set_length(len);
                            }

                            resp.into_body()
                                .map_err(|e| eprintln!("failed to read body: {}", e))
                                .fold((prog, file, tmp), move |(prog, mut file, tmp), chunk| {
                                    file.write(&chunk)
                                        .map(|len| {
                                            prog.inc(len as u64);
                                            prog.set_message(&format!("downloading {}", uri));
                                            (prog, file, tmp)
                                        }).map_err(|e| eprintln!("failed to write chunk: {}", e))
                                }).and_then(move |(prog, _, tmp)| {
                                    prog.finish_with_message(&format!(
                                        "downloaded {}",
                                        tmp.file_name().unwrap().to_string_lossy()
                                    ));
                                    let dest =
                                        prefix.join(SOURCES_SUBDIR).join(tmp.file_name().unwrap());
                                    tokio::fs::rename(tmp, dest.clone())
                                        .map(move |_| dest)
                                        .map_err(|e| println!("failed to rename file: {}", e))
                                })
                        })
                }),
        );

        Ok(Download { progress, future })
    }

    pub fn query(&self, _hash: String) -> Result<Option<PathBuf>, IoError> {
        unimplemented!()
    }
}

impl Debug for StoreDirectory {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_struct(stringify!(StoreDirectory))
            .field("prefix", &self.prefix)
            .field("conn", &stringify!(SqliteConnection))
            .finish()
    }
}

pub struct Download {
    progress: Arc<ProgressBar>,
    future: Box<Future<Item = PathBuf, Error = ()> + Send>,
}

fn new_store_transaction<T, F>(dir: &Path, run_txn: F) -> Result<T, CreationError>
where
    F: Fn(&Path) -> Result<T, CreationError>,
{
    let out = run_txn(&dir)?;

    let walker = WalkBuilder::new(&dir).ignore(false).build_parallel();
    let dir = Arc::new(dir.to_path_buf());
    let walker_err = Arc::new(Mutex::new(None));
    walker.run(|| {
        let dir = dir.clone();
        let walker_err = walker_err.clone();
        Box::new(move |entry| {
            let item = match entry {
                Ok(item) => item,
                Err(err) => {
                    *walker_err.lock().unwrap_or_else(|e| e.into_inner()) =
                        Some(CreationError::ReadDirectory(err));
                    return WalkState::Quit;
                }
            };

            let mut perms = match fs::metadata(item.path()) {
                Ok(meta) => meta.permissions(),
                Err(err) => {
                    *walker_err.lock().unwrap_or_else(|e| e.into_inner()) =
                        Some(CreationError::ReadPermissions(err));
                    return WalkState::Quit;
                }
            };

            if cfg!(unix) {
                use std::os::unix::fs::PermissionsExt;
                let mode = perms.mode();
                perms.set_mode(mode | 0o1000);
            }

            if item.depth() > 1 && !item.path().starts_with(dir.join(VAR_SUBDIR)) {
                perms.set_readonly(true);
                let zero = FileTime::zero();
                if let Err(err) = filetime::set_symlink_file_times(item.path(), zero, zero) {
                    *walker_err.lock().unwrap_or_else(|e| e.into_inner()) =
                        Some(CreationError::WriteTimestamps(err));
                    return WalkState::Quit;
                }
            }

            if let Err(err) = fs::set_permissions(item.path(), perms) {
                *walker_err.lock().unwrap_or_else(|e| e.into_inner()) =
                    Some(CreationError::WritePermissions(err));
                return WalkState::Quit;
            }

            WalkState::Continue
        })
    });

    let mut guard = Arc::try_unwrap(walker_err)
        .expect("Arc still has multiple owners")
        .into_inner()
        .unwrap_or_else(|e| e.into_inner());

    if let Some(err) = guard.take() {
        return Err(err);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let executor = runtime.executor();

        let dir = StoreDirectory::open(concat!(env!("CARGO_MANIFEST_DIR"), "/store").into())
            .expect("store");
        let download1 = dir
            .download_source(
                "http://file-examples.com/wp-content/uploads/2017/02/zip_10MB.zip"
                    .parse()
                    .expect("uri"),
            ).expect("download1");
        let download2 = dir
            .download_source(
                "http://file-examples.com/wp-content/uploads/2017/02/zip_9MB.zip"
                    .parse()
                    .expect("uri"),
            ).expect("download2");

        // let jobs: Vec<Box<Future<Item = _, Error = _> + Send>> = vec![
        let jobs: Vec<Box<Future<Item = _, Error = _> + Send>> = vec![
            Box::new(download1.future.map(|_| ())),
            Box::new(download2.future.map(|_| ())),
        ];

        executor.spawn(future::join_all(jobs).map(|_| ()));

        dir.progress.join().expect("error joining progress");

        runtime
            .shutdown_on_idle()
            .wait()
            .expect("error shutting down");
    }
}
