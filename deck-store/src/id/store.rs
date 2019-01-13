use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::Path;
use std::str::FromStr;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};
use url::{ParseError as UrlError, Url};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    NotStoreId,
    Url(UrlError),
    UnsupportedPrefix(String),
    UnsupportedScheme(String),
    MissingHost,
    MissingContainerInfo,
    MissingRequiredQueryPair(String),
    UnknownFragment(String),
    UnknownQueryPair(String, String),
}

impl Display for ParseError {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        use self::ParseError::*;
        match *self {
            NotStoreId => write!(fmt, "expected store ID with the form `prefix+url`"),
            Url(ref e) => write!(fmt, "failed to parse URL: {}", e),
            UnsupportedPrefix(ref p) => write!(fmt, "unsupported prefix `{}+`", p),
            UnsupportedScheme(ref s) => write!(fmt, "unsupported URL scheme `{}://`", s),
            MissingHost => write!(fmt, "URL scheme requires a host"),
            MissingContainerInfo => write!(
                fmt,
                "Docker store ID missing a `?container_id=` or `?container_name=`"
            ),
            MissingRequiredQueryPair(ref k) => {
                write!(fmt, "missing required query pair `?{}=...`", k)
            }
            UnknownFragment(ref f) => write!(fmt, "unknown URL fragment `#{}`", f),
            UnknownQueryPair(ref k, ref v) => write!(fmt, "unknown query pair `?{}={}`", k, v),
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            ParseError::Url(ref e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Kind {
    /// A local directory store, either in single-user or multi-user modes.
    Local,
    /// A remote SSH store.
    Ssh,
    /// A remote Docker store.
    Docker(DockerContainer),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DockerContainer {
    Id(String),
    Name(String),
}

/// `local+file:///deck/store`
/// `ssh+ssh://[user@]host[:port]`
/// `docker+unix:///var/run/docker.sock?container_id=foo&container_name=bar&user=baz`
/// `docker+https://host[:port]?container_id=foo&container_name=bar&user=baz`
/// `docker+ssh://host[:port]?container_id=foo&container_name=bar&user=baz`
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StoreId {
    url: Url,
    kind: Kind,
}

impl StoreId {
    pub fn from_url(url: &str) -> Result<Self, ParseError> {
        let mut parts = url.splitn(2, '+');

        let prefix = parts.next().ok_or(ParseError::NotStoreId)?;
        let url: Url = parts
            .next()
            .ok_or(ParseError::NotStoreId)
            .and_then(|url| url.parse().map_err(ParseError::Url))?;

        match prefix {
            "local" => match url.scheme() {
                "file" => Self::for_local(url.path().as_ref()),
                s => Err(ParseError::UnsupportedScheme(s.into())),
            },
            "ssh" => Self::for_ssh(url),
            "docker" => Self::for_docker(url),
            p => Err(ParseError::UnsupportedPrefix(p.into())),
        }
    }

    pub fn for_local(path: &Path) -> Result<Self, ParseError> {
        let url = Url::from_directory_path(path).unwrap();

        if let Some(frag) = url.fragment() {
            return Err(ParseError::UnknownFragment(frag.into()));
        } else if let Some((k, v)) = url.query_pairs().next() {
            return Err(ParseError::UnknownQueryPair(k.into_owned(), v.into_owned()));
        }

        Ok(StoreId {
            url,
            kind: Kind::Local,
        })
    }

    pub fn for_ssh(mut url: Url) -> Result<Self, ParseError> {
        if url.scheme() != "ssh" {
            let scheme = url.scheme();
            return Err(ParseError::UnsupportedScheme(scheme.into()));
        } else if url.host().is_none() {
            return Err(ParseError::MissingHost);
        }

        url.set_fragment(None);
        url.set_query(None);

        Ok(StoreId {
            url,
            kind: Kind::Ssh,
        })
    }

    pub fn for_docker(url: Url) -> Result<Self, ParseError> {
        match url.scheme() {
            "unix" | "https" | "ssh" => {}
            s => return Err(ParseError::UnsupportedScheme(s.into())),
        }

        if url.host().is_none() {
            return Err(ParseError::MissingHost);
        }

        let user = url
            .query_pairs()
            .find(|(k, _)| k == "user")
            .map(|(_, v)| v.into_owned())
            .ok_or(ParseError::MissingRequiredQueryPair("user".into()))?;

        let mut new_url = url.clone();
        new_url.set_fragment(None);
        new_url.set_query(None);
        new_url.query_pairs_mut().append_pair("user", &user);

        let container = url
            .query_pairs()
            .filter(|(k, _)| k != "user")
            .next()
            .ok_or(ParseError::MissingContainerInfo)
            .and_then(|(k, v)| match k.as_ref() {
                "container_id" => {
                    new_url.query_pairs_mut().append_pair("container_id", &v);
                    Ok(DockerContainer::Id(v.into_owned()))
                }
                "container_name" => {
                    new_url.query_pairs_mut().append_pair("container_name", &v);
                    Ok(DockerContainer::Name(v.into_owned()))
                }
                k => Err(ParseError::UnknownQueryPair(k.into(), v.into_owned())),
            })?;

        Ok(StoreId {
            url: new_url,
            kind: Kind::Docker(container),
        })
    }

    #[inline]
    pub fn is_local(&self) -> bool {
        self.kind == Kind::Local
    }

    #[inline]
    pub fn is_ssh(&self) -> bool {
        self.kind == Kind::Ssh
    }

    #[inline]
    pub fn is_docker(&self) -> bool {
        match self.kind {
            Kind::Docker(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn as_url(&self) -> &Url {
        &self.url
    }
}

impl Display for StoreId {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{}", self.url)
    }
}

impl FromStr for StoreId {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        StoreId::from_url(s)
    }
}

impl<'de> Deserialize<'de> for StoreId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        StoreId::from_str(&s).map_err(|err| E::custom(err.to_string()))
    }
}

impl Serialize for StoreId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_local_urls() {
        let expected = Ok(StoreId {
            url: "file:///deck/store/".parse().expect("Failed to parse URL"),
            kind: Kind::Local,
        });

        let actual = "local+file:///deck/store".parse();
        assert_eq!(actual, expected);

        let actual = "local+file:///deck/store#foo".parse();
        assert_eq!(actual, expected);

        let actual = "local+file:///deck/store?foo=bar".parse();
        assert_eq!(actual, expected);

        let result = "local+http://www.example.com".parse::<StoreId>();
        assert!(result.is_err());
    }

    #[test]
    fn parses_ssh_urls() {
        let actual = "ssh+ssh://server".parse();
        let expected = Ok(StoreId {
            url: "ssh://server".parse().expect("Failed to parse URL"),
            kind: Kind::Ssh,
        });
        assert_eq!(actual, expected);

        let actual = "ssh+ssh://user@server".parse();
        let expected = Ok(StoreId {
            url: "ssh://user@server".parse().expect("Failed to parse URL"),
            kind: Kind::Ssh,
        });
        assert_eq!(actual, expected);

        let actual = "ssh+ssh://user@server:22".parse();
        let expected = Ok(StoreId {
            url: "ssh://user@server:22".parse().expect("Failed to parse URL"),
            kind: Kind::Ssh,
        });
        assert_eq!(actual, expected);

        let actual = "ssh+ssh://user@server:22#fragment".parse();
        assert_eq!(actual, expected);

        let actual = "ssh+ssh://user@server:22?foo=bar".parse();
        assert_eq!(actual, expected);

        let result = "ssh+http://www.example.com".parse::<StoreId>();
        assert!(result.is_err());
    }

    #[test]
    fn parses_docker_urls() {
        let actual = "docker+ssh://user@host:22?user=foo&container_name=gcr.io/org/bar".parse();
        let expected = Ok(StoreId {
            url: "ssh://user@host:22?user=foo&container_name=gcr.io%2Forg%2Fbar"
                .parse()
                .expect("Failed to parse URL"),
            kind: Kind::Docker(DockerContainer::Name("gcr.io/org/bar".into())),
        });
        assert_eq!(actual, expected);

        let actual = "docker+https://host/?user=foo&container_id=0123456789ab".parse();
        let expected = Ok(StoreId {
            url: "https://host/?user=foo&container_id=0123456789ab"
                .parse()
                .expect("Failed to parse URL"),
            kind: Kind::Docker(DockerContainer::Id("0123456789ab".into())),
        });
        assert_eq!(actual, expected);

        let actual =
            "docker+https://host/?user=foo&container_id=0123456789ab&container_name=slug".parse();
        assert_eq!(actual, expected);

        let actual = "docker+https://host/?user=foo&container_id=0123456789ab&bar=hello".parse();
        assert_eq!(actual, expected);

        let result = "docker+unix:///var/run/docker.sock?user=foo".parse::<StoreId>();
        assert!(result.is_err());

        let result = "docker+unix:///var/run/docker.sock".parse::<StoreId>();
        assert!(result.is_err());

        let result = "docker+unix:///var/run/docker.sock#fragment".parse::<StoreId>();
        assert!(result.is_err());

        let result = "docker+unix:///var/run/docker.sock?foo=bar".parse::<StoreId>();
        assert!(result.is_err());

        let result = "docker+ftp://ftp.example.com".parse::<StoreId>();
        assert!(result.is_err());
    }
}
