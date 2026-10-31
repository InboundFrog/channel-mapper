//! tvmap: renumber channels in a Samsung Tizen channel-list export.

pub mod archive;
pub mod db;
pub mod mapfile;
pub mod plan;

use anyhow::{bail, Result};
use archive::ChannelArchive;
use db::Service;
use plan::Plan;
use std::path::Path;

/// Open an export and read every broadcast service in it.
pub fn load(zip: &Path) -> Result<(ChannelArchive, Vec<Service>)> {
    let archive = ChannelArchive::open(zip)?;
    let mut services = Vec::new();
    for d in &archive.dbs {
        services.extend(db::read_services(&d.open_ro()?, &d.entry)?);
    }
    Ok((archive, services))
}

/// Write the plan into a new export at `out`, starting from a fresh copy of
/// the original `zip` each time. Refuses to write if two services would share
/// a number. Returns the number of services changed.
pub fn save(zip: &Path, plan: &Plan, out: &Path) -> Result<usize> {
    let archive = ChannelArchive::open(zip)?;
    let conflicts = plan.conflicts();
    if !conflicts.is_empty() {
        let list: Vec<String> = conflicts
            .iter()
            .map(|(n, rows)| {
                let names: Vec<&str> = rows
                    .iter()
                    .map(|&i| plan.rows[i].svc.name.as_str())
                    .collect();
                format!("{n}: {}", names.join(", "))
            })
            .collect();
        bail!(
            "duplicate channel numbers, not saving:\n  {}",
            list.join("\n  ")
        );
    }
    let mut total = 0;
    for (entry, changes) in plan.changes() {
        let d = archive
            .dbs
            .iter()
            .find(|d| d.entry == entry)
            .expect("change refers to a loaded db");
        total += db::write_settings(&mut d.open_rw()?, &changes)?;
    }
    archive.save(out)?;
    Ok(total)
}
