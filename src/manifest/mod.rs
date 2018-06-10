pub use ron::de::Error as ParseError;

use std::borrow::Cow;
use std::str::FromStr;

use ron;
use semver::Version;

use self::build_system::BuildSystem;
use self::deps::Dependencies;
use self::features::Features;
use self::source::Source;
use license::License;

mod build_system;
mod deps;
mod features;
mod source;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "Package")]
pub struct Manifest<'m> {
    name: Cow<'m, str>,
    version: Version,
    license: License,
    description: Cow<'m, str>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    authors: Vec<Cow<'m, str>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    maintainers: Vec<Cow<'m, str>>,

    #[serde(default)]
    features: Features<'m>,
    #[serde(default)]
    dependencies: Dependencies<'m>,
    source: Source<'m>,
    build_system: BuildSystem<'m>,
}

impl<'m> Manifest<'m> {
    pub fn parse<S: AsRef<str>>(string: S) -> Result<Self, ParseError> {
        ron::de::from_str(string.as_ref())
    }
}

impl<'m> FromStr for Manifest<'m> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RON_MANIFEST: &str = r##"
        Package(
            name: "my-package",
            version: "1.0.0",
            license: "MIT OR Apache-2.0",
            description: "Some thing I tried",
            authors: ["Gordon Freeman <gordon.freeman@blackmesa.gov>"],
            maintainers: ["Barney Calhoun <barney.calhoun@blackmesa.gov>"],
            features: {
                "default": [],
            },
            dependencies: {
                "thing": "1.0.0",
                "other-thing": (
                    version: "1.0.0",
                    features: ["mything"],
                ),
            },
            source: (
                method: FetchGit(
                    repo: "https://github.com/blackmesa/my-package",
                    branch: "thing",
                    sha256: "1234567890abcdef",
                ),
            ),
            build_system: Gnu(
                configure_flags: ["--enable-silent-rules"],
                modify_phases: [
                    AddAfter(
                        phase: Configure,
                        do: [
                            Replace("etc/config", "# enable-thing = true", "enable-thing = true")
                        ]
                    ),
                    Delete(Check),
                ]
            )
        )
    "##;

    #[test]
    fn it_works() {
        let _manifest: Manifest = RON_MANIFEST.parse().unwrap();
    }
}
