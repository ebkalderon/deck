use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::marker::PhantomData;

use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

#[derive(Debug, Eq, PartialEq)]
pub struct Features<'m>(BTreeMap<String, BTreeSet<Cow<'m, str>>>);

impl<'m> Default for Features<'m> {
    fn default() -> Self {
        let mut feat = BTreeMap::new();
        feat.insert("default".to_string(), BTreeSet::new());
        Features(feat)
    }
}

impl<'m, 'de> Deserialize<'de> for Features<'m> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Default)]
        struct FeaturesVisitor<'m>(PhantomData<&'m ()>);

        impl<'m, 'de> de::Visitor<'de> for FeaturesVisitor<'m> {
            type Value = Features<'m>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a map of features")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut inner = BTreeMap::new();
                while let Some((key, value)) = map.next_entry()? {
                    inner.insert(key, value);
                }

                Ok(Features(inner))
            }
        }

        deserializer.deserialize_map(FeaturesVisitor::default())
    }
}

impl<'m> Serialize for Features<'m> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let Features(ref inner) = *self;
        inner.serialize(serializer)
    }
}
