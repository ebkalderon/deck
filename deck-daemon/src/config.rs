#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    build_group: Option<String>,
    max_builds: Option<u32>,
    trusted_users: Option<Vec<String>>,
}
