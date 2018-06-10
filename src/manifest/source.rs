use std::borrow::Cow;

use url::Url;
use url_serde;

#[derive(Debug, Deserialize, Serialize)]
pub struct Source<'m> {
    method: Method<'m>,
    #[serde(default)]
    #[serde(skip_serializing_if = "str::is_empty")]
    saved_name: Cow<'m, str>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    patches: Vec<Source<'m>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Method<'m> {
    FetchGit {
        #[serde(with = "url_serde")]
        repo: Url,
        sha256: Cow<'m, str>,
        #[serde(default)]
        #[serde(skip_serializing_if = "str::is_empty")]
        branch: Cow<'m, str>,
        #[serde(default)]
        #[serde(skip_serializing_if = "str::is_empty")]
        rev: Cow<'m, str>,
    },
    FetchUrl {
        #[serde(with = "url_serde")]
        url: Url,
        sha256: Cow<'m, str>,
    }
}
