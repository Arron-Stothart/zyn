use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageJsonWire {
    pub name: Option<String>,
    pub version: Option<String>,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
    #[serde(default)]
    pub dev_dependencies: BTreeMap<String, String>,
    #[serde(default)]
    pub optional_dependencies: BTreeMap<String, String>,
    #[serde(default)]
    pub peer_dependencies: BTreeMap<String, String>,
}

impl PackageJsonWire {
    pub fn from_str(source: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(source)
    }
}
