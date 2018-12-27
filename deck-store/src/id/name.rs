use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Name(String);

impl Name {
    pub fn new<S: Into<String>>(name: S) -> Result<Name, ()> {
        let s = name.into();
        if s.is_empty() {
            return Err(());
        }

        let allowed_chars = s
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.');

        if !allowed_chars {
            return Err(());
        }

        Ok(Name(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Name {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_send_sync() {
        fn verify<T: Send + Sync>() {}
        verify::<Name>();
    }

    #[test]
    fn parse_valid_names() {
        Name::new("foo-bar").expect("Failed to parse valid name with hyphen!");
        Name::new("foo_bar").expect("Failed to parse valid name with underscore!");
        Name::new("f0-o_B4.r").expect("Failed to parse valid name with mixed chars!");
    }

    #[test]
    fn reject_invalid_names() {
        Name::new("foo bar").expect_err("Failed to reject name with space");
        Name::new("/foo/bar").expect_err("Failed to reject name with path-like slashes");
        Name::new("foo!@#$%^&*(){}+?<>'\"").expect_err("Failed to reject name with special chars!");
    }

    #[test]
    fn reject_empty_name() {
        Name::new("").expect_err("Failed to reject empty name");
    }
}
