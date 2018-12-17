use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    InvalidFormat,
    MissingOs,
    UnknownArch(UnknownArch),
    UnknownOs(UnknownOs),
}

impl Display for ParseError {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            ParseError::InvalidFormat => write!(fmt, "invalid target triple"),
            ParseError::MissingOs => write!(fmt, "missing OS and vendor"),
            ParseError::UnknownArch(ref e) => write!(fmt, "{}", e),
            ParseError::UnknownOs(ref e) => write!(fmt, "{}", e),
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            ParseError::UnknownArch(ref e) => Some(e),
            ParseError::UnknownOs(ref e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Platform {
    pub target_arch: Arch,
    pub target_os: Os,
}

impl Display for Platform {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{}-{}", self.target_arch, self.target_os)
    }
}

impl FromStr for Platform {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.trim().splitn(2, '-');
        let target_arch: Arch = tokens
            .next()
            .ok_or(ParseError::InvalidFormat)
            .and_then(|arch| arch.parse().map_err(ParseError::UnknownArch))?;

        let target_os: Os = tokens
            .next()
            .ok_or(ParseError::MissingOs)
            .and_then(|os| os.parse().map_err(ParseError::UnknownOs))?;

        if tokens.count() != 0 {
            return Err(ParseError::InvalidFormat);
        }

        Ok(Platform {
            target_arch,
            target_os,
        })
    }
}

impl<'de> Deserialize<'de> for Platform {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PlatformVisitor;

        impl<'de> Visitor<'de> for PlatformVisitor {
            type Value = Platform;

            fn expecting(&self, fmt: &mut Formatter) -> FmtResult {
                fmt.write_str("a target triple, e.g. x86_64-unknown-linux")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Platform::from_str(value).map_err(|err| E::custom(err.to_string()))
            }
        }

        deserializer.deserialize_str(PlatformVisitor)
    }
}

impl Serialize for Platform {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownArch(String);

impl Display for UnknownArch {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "unknown CPU architecture `{}`", self.0)
    }
}

impl Error for UnknownArch {
    fn cause(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Arch {
    I686,
    X86_64,
}

impl Display for Arch {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            Arch::I686 => write!(fmt, "i686"),
            Arch::X86_64 => write!(fmt, "x86_64"),
        }
    }
}

impl FromStr for Arch {
    type Err = UnknownArch;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "i686" => Ok(Arch::I686),
            "x86_64" => Ok(Arch::X86_64),
            s => Err(UnknownArch(s.to_string())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownOs(String);

impl Display for UnknownOs {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "unknown operating system `{}`", self.0)
    }
}

impl Error for UnknownOs {
    fn cause(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Os {
    Darwin,
    FreeBsd,
    Linux,
    NetBsd,
    Windows,
}

impl Display for Os {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            Os::Darwin => write!(fmt, "apple-darwin"),
            Os::FreeBsd => write!(fmt, "unknown-freebsd"),
            Os::Linux => write!(fmt, "unknown-linux"),
            Os::NetBsd => write!(fmt, "unknown-netbsd"),
            Os::Windows => write!(fmt, "pc-windows"),
        }
    }
}

impl FromStr for Os {
    type Err = UnknownOs;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "apple-darwin" => Ok(Os::Darwin),
            "unknown-freebsd" => Ok(Os::FreeBsd),
            "unknown-linux" => Ok(Os::Linux),
            "unknown-netbsd" => Ok(Os::NetBsd),
            "pc-windows" => Ok(Os::Windows),
            s => Err(UnknownOs(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_triples() {
        let actual = "x86_64-unknown-linux".parse();
        let expected = Ok(Platform {
            target_arch: Arch::X86_64,
            target_os: Os::Linux,
        });
        assert_eq!(actual, expected);

        let actual = "x86_64-pc-windows".parse();
        let expected = Ok(Platform {
            target_arch: Arch::X86_64,
            target_os: Os::Windows,
        });
        assert_eq!(actual, expected);

        let actual = "x86_64-apple-darwin".parse();
        let expected = Ok(Platform {
            target_arch: Arch::X86_64,
            target_os: Os::Darwin,
        });
        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_invalid_triples() {
        let result = "i686- unknown-freebsd".parse::<Platform>();
        assert!(result.is_err());

        let result = "i686 -unknown-freebsd".parse::<Platform>();
        assert!(result.is_err());

        let result = "pc-windows-x86_64".parse::<Platform>();
        assert!(result.is_err());
    }

    #[test]
    fn tolerates_leading_trailing_spaces() {
        let expected = Ok(Platform {
            target_arch: Arch::X86_64,
            target_os: Os::Linux,
        });

        let actual = "x86_64-unknown-linux   ".parse();
        assert_eq!(actual, expected);

        let actual = "   x86_64-unknown-linux".parse();
        assert_eq!(actual, expected);

        let actual = "   x86_64-unknown-linux   ".parse();
        assert_eq!(actual, expected);
    }
}
