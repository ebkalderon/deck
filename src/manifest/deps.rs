use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

use semver::VersionReq;
use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Dependency<'m> {
    Simple(Cow<'m, str>),
    Complex {
        version: VersionReq,
        #[serde(default)]
        default_features: bool,
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        features: Vec<Cow<'m, str>>,
        #[serde(default)]
        optional: bool,
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Dependencies<'m>(BTreeMap<String, Dependency<'m>>);

impl<'m, 'de> Deserialize<'de> for Dependencies<'m> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Default)]
        struct DependenciesVisitor<'m>(PhantomData<&'m ()>);

        impl<'m, 'de> de::Visitor<'de> for DependenciesVisitor<'m> {
            type Value = Dependencies<'m>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a map of dependencies")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut inner = BTreeMap::new();
                while let Some((key, value)) = map.next_entry()? {
                    inner.insert(key, value);
                }

                Ok(Dependencies(inner))
            }
        }

        deserializer.deserialize_map(DependenciesVisitor::default())
    }
}

impl<'m> Serialize for Dependencies<'m> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let Dependencies(ref inner) = *self;
        inner.serialize(serializer)
    }
}
