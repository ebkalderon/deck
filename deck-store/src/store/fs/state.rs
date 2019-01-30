use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;
use std::sync::Arc;

use futures::future::{self, Loop};
use futures::{Async, Future, Poll};
use futures_locks::RwLock;
use tokio::fs;

use super::dir::{Directory, DirectoryFuture, FetchStream, ReadFuture, WriteFuture};
// use super::fetcher::Fetcher;
use crate::id::FilesystemId;

const TEMP_DIR_NAME: &str = "tmp";

type WriteQueue<I> = Arc<RwLock<HashMap<I, BlockingFuture<I>>>>;

#[derive(Debug)]
pub struct State<D: Directory> {
    directory: Arc<D>,
    write_queue: WriteQueue<D::Id>,
}

impl<D> State<D>
where
    D: Directory + 'static,
    D::Id: 'static,
    D::Input: 'static,
    D::Output: 'static,
{
    pub fn new(directory: D) -> Self {
        State {
            directory: Arc::new(directory),
            write_queue: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn contains(&self, prefix: &Path, id: &D::Id) -> bool {
        let path = prefix.join(D::NAME).join(id.to_path());
        path.exists()
    }

    pub fn read(&self, prefix: &Path, id: &D::Id) -> ReadFuture<D::Output> {
        let entry_id = id.clone();
        let write_queue = self.write_queue.clone();

        let block_if_writing = future::loop_fn((entry_id, write_queue), |(id, write_queue)| {
            let reading = write_queue.read();
            reading.and_then(|writers| -> Box<dyn Future<Item = _, Error = _> + Send> {
                if let Some(block) = writers.get(&id).cloned() {
                    Box::new(block.map(|id| Loop::Continue((id, write_queue))))
                } else {
                    Box::new(future::ok(Loop::Break(id)))
                }
            })
        });

        let path = prefix.join(D::NAME).join(id.to_path());
        let read_data = self.directory.read(&path, &id);
        let output = block_if_writing.and_then(move |_| read_data);

        DirectoryFuture::new(output)
    }

    pub fn write(&self, prefix: &Path, input: D::Input) -> WriteFuture<D::Id, D::Output> {
        // Since the `D::Id` of a given `D::Input` is not known ahead of time, we compute a
        // temporary one here and use it to mark ourselves as writing. A new `D::Id`, which may be
        // different from the temporary one, will be returned from `Directory::write()` along with
        // the `D::Output`.
        let compute_temp_id = self.directory.precompute_id(&input);

        // TODO: If `<prefix>/tmp/<temp_id>` or `<prefix>/D::NAME/<temp_id>` exists, return early.

        let write_queue = self.write_queue.clone();
        let mark_as_writing = compute_temp_id.and_then(|temp_id| {
            future::loop_fn((temp_id, write_queue), |(id, write_queue)| {
                let locking = write_queue.write();
                locking.and_then(
                    |mut writers| -> Box<dyn Future<Item = _, Error = _> + Send> {
                        if let Some(block) = writers.get(&id).cloned() {
                            Box::new(block.map(|id| Loop::Continue((id, write_queue))))
                        } else {
                            let block = BlockingFuture::wait_for(id.clone(), write_queue.clone());
                            writers.insert(id.clone(), block);
                            Box::new(future::ok(Loop::Break(id)))
                        }
                    },
                )
            })
        });

        let write_data = {
            let directory = self.directory.clone();
            let temp_prefix = prefix.join(TEMP_DIR_NAME);

            mark_as_writing.and_then(move |temp_id: D::Id| {
                let temp_path = temp_prefix.join(temp_id.to_path());
                directory
                    .write(&temp_path, input)
                    .map(|(new_id, out)| (temp_path, temp_id, new_id, out))
            })
        };

        let final_prefix = prefix.join(D::NAME);
        let rename_path = write_data.and_then(move |(temp_path, temp_id, new_id, out)| {
            let final_path = final_prefix.join(new_id.to_path());
            println!("renaming {:?} -> {:?}", temp_path, final_path);
            fs::rename(temp_path, final_path)
                .map(|_| (temp_id, new_id, out))
                .map_err(|e| eprintln!("failed to rename: {}", e))
        });

        let check_writers = self.write_queue.write();
        let output = rename_path
            .inspect(|_| /* TODO: Need to normalize permissions using known path here. */ ())
            .inspect(|_| /* TODO: Need to register paths in database here. */ ())
            .and_then(|(temp_id, new_id, out)| {
                check_writers.map(move |w| (w, temp_id, new_id, out))
            })
            .map(|(mut writers, temp_id, new_id, output)| {
                writers.remove(&temp_id);
                (new_id, output)
            });

        DirectoryFuture::new(output)
    }

    // pub fn fetch<F: Fetcher>(&self, prefix: &Path, fetcher: F) -> FetchStream<D::Id> {
    //     // How do we pre-compute the ID? The method requires a `D::Input`.
    //     unimplemented!()
    // }
}

#[derive(Clone, Debug)]
#[must_use = "futures do nothing unless polled"]
struct BlockingFuture<I: Clone + Eq + Hash> {
    id: Option<I>,
    write_queue: Option<WriteQueue<I>>,
}

impl<I: Clone + Eq + Hash> BlockingFuture<I> {
    fn wait_for(id: I, write_queue: WriteQueue<I>) -> Self {
        BlockingFuture {
            id: Some(id),
            write_queue: Some(write_queue),
        }
    }
}

impl<I: Clone + Eq + Hash> Future for BlockingFuture<I> {
    type Item = I;
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let id = self.id.take().expect("ID was empty!");
        let table = self.write_queue.take().expect("WriteQueue<I> was empty!");

        if let Ok(ref write_queue) = table.try_read() {
            if write_queue.contains_key(&id) {
                self.id = Some(id);
                self.write_queue = Some(table);
                Ok(Async::NotReady)
            } else {
                Ok(Async::Ready(id))
            }
        } else {
            self.id = Some(id);
            self.write_queue = Some(table);
            Ok(Async::NotReady)
        }
    }
}
