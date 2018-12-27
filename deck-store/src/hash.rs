use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::BufRead;
use std::str::FromStr;

use blake2::digest::{Input, VariableOutput};
use blake2::VarBlake2b;
use data_encoding::BASE32_NOPAD;
use rand::{self, RngCore};

const HASH_LENGTH: usize = 20;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Hash([u8; HASH_LENGTH]);

impl Hash {
    pub fn compute() -> Builder {
        Builder::new()
    }

    pub fn random() -> Self {
        let mut buffer = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut buffer);
        Hash::compute().input(buffer).finish()
    }

    pub fn from_reader<R: BufRead>(reader: &mut R) -> Result<Hash, ()> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).map_err(|_| ())?;
        let hash = Builder::new().input(buf).finish();
        Ok(hash)
    }
}

impl Display for Hash {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        let encoded = BASE32_NOPAD.encode(&self.0);
        write!(fmt, "{}", encoded.to_lowercase())
    }
}

impl FromStr for Hash {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded = BASE32_NOPAD
            .decode(s.to_uppercase().as_bytes())
            .map_err(|_| ())?;

        if s.len() == BASE32_NOPAD.encode_len(HASH_LENGTH) {
            let mut buffer = [0u8; HASH_LENGTH];
            buffer.copy_from_slice(decoded.as_slice());
            Ok(Hash(buffer))
        } else {
            Err(())
        }
    }
}

#[derive(Debug)]
pub struct Builder {
    hasher: VarBlake2b,
}

impl Builder {
    fn new() -> Self {
        Builder {
            hasher: VarBlake2b::new(HASH_LENGTH).expect("HASH_LENGTH is an invalid value"),
        }
    }

    pub fn input<B: AsRef<[u8]>>(mut self, bytes: B) -> Self {
        self.hasher.input(bytes);
        self
    }

    pub fn finish(self) -> Hash {
        let mut output = [0u8; HASH_LENGTH];
        self.hasher.variable_result(|b| output.copy_from_slice(b));
        Hash(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_send_and_sync() {
        fn check_send_and_sync<T: Send + Sync>() {}
        check_send_and_sync::<Hash>();
    }

    #[test]
    fn parse_roundtrip() {
        let original = Hash::random();
        let text_form = original.to_string();

        let parsed: Hash = text_form.parse().expect("Failed to parse hash from text");
        assert_eq!(original, parsed);
    }

    #[test]
    fn parse_upper_and_lower_case() {
        Hash::from_str("fc3j3vub6kodu4jtfoakfs5xhumqi62m").expect("Failed to parse lowercase hash");
        Hash::from_str("FC3J3VUB6KODU4JTFOAKFS5XHUMQI62M").expect("Failed to parse uppercase hash");
    }

    #[test]
    fn print_lower_case() {
        let hash = Hash::from_str("fc3j3vub6kodu4jtfoakfs5xhumqi62m").expect("Failed to parse");
        let s = hash.to_string();
        assert!(s.chars().all(|c| c.is_numeric() || c.is_lowercase()));
    }

    #[test]
    fn reject_invalid_hashes() {
        Hash::from_str("1234567890").expect_err("Failed to reject non-hash value");
        Hash::from_str("gezdgnbvgy3tqojq").expect_err("Failed to reject base32 of non-hash value");
        Hash::from_str("28b69dd681f29c3a71332b80a2cbb73d1947b4c")
            .expect_err("Failed to reject non-base32 valid hash");
    }
}
