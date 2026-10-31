//! The channel map: which broadcast service gets which number.

use crate::db::ServiceKey;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Built-in map for Perth, WA.
pub const PERTH: &str = include_str!("../maps/perth.toml");

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MapFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(default, rename = "channel")]
    pub channels: Vec<MapEntry>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapEntry {
    pub onid: i64,
    pub tsid: i64,
    pub sid: i64,
    /// Name the service had when the map was made.
    pub name: String,
    /// Channel number. `None` keeps whatever the TV assigned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<i64>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub hidden: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub skip: bool,
}

impl MapEntry {
    pub fn key(&self) -> ServiceKey {
        ServiceKey {
            onid: self.onid,
            tsid: self.tsid,
            sid: self.sid,
        }
    }
}

impl MapFile {
    pub fn parse(text: &str) -> Result<Self> {
        let m: MapFile = toml::from_str(text).context("parsing channel map")?;
        m.validate()?;
        Ok(m)
    }

    pub fn load(path: &Path) -> Result<Self> {
        let text =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        Self::parse(&text).with_context(|| format!("in {}", path.display()))
    }

    pub fn perth() -> Self {
        Self::parse(PERTH).expect("built-in Perth map is valid")
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let header =
            "# tvmap channel map. Keyed on onid/tsid/sid; `name` is used to flag changes.\n\n";
        std::fs::write(path, format!("{header}{}", toml::to_string_pretty(self)?))
            .with_context(|| format!("writing {}", path.display()))
    }

    fn validate(&self) -> Result<()> {
        let mut seen = std::collections::HashMap::new();
        for e in &self.channels {
            if let Some(n) = e.number {
                anyhow::ensure!(
                    (1..=9999).contains(&n),
                    "{} ({}): number {n} out of range 1-9999",
                    e.name,
                    e.key()
                );
            }
            if let Some(prev) = seen.insert(e.key(), &e.name) {
                anyhow::bail!("service {} listed twice ({prev} / {})", e.key(), e.name);
            }
        }
        Ok(())
    }
}
