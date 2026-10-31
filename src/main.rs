use anyhow::{bail, Result};
use clap::{Args, Parser, Subcommand};
use std::path::{Path, PathBuf};
use tvmap::mapfile::MapFile;
use tvmap::plan::{Plan, Row};

mod tui;

/// Renumber channels in a Samsung TV channel-list export (Channel_list_*.zip).
///
/// Export the list from the TV to a FAT32 USB stick (Settings > Broadcasting >
/// Expert Settings > Transfer Channel List, PIN 0000), run this on it, then
/// import the new file the same way.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Show every service and what the map would do to it.
    List(Common),
    /// Apply the map without the interactive editor.
    Apply {
        #[command(flatten)]
        common: Common,
        /// Output zip. Default: tvmap-out/<same file name>.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Show what would change, write nothing.
        #[arg(long)]
        dry_run: bool,
        /// Also save the resulting map, to re-apply after a rescan.
        #[arg(long)]
        save_map: Option<PathBuf>,
    },
    /// Interactive editor (TUI).
    Edit {
        #[command(flatten)]
        common: Common,
        /// Output zip. Default: tvmap-out/<same file name>.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Where to save the map on write. Default: tvmap-out/channel-map.toml.
        #[arg(long)]
        save_map: Option<PathBuf>,
    },
}

#[derive(Args)]
struct Common {
    /// The Channel_list_*.zip exported from the TV.
    zip: PathBuf,
    /// Channel map (TOML). Default: built-in Perth map.
    #[arg(short, long, conflicts_with = "empty")]
    map: Option<PathBuf>,
    /// Start from an empty map instead of the built-in one.
    #[arg(long)]
    empty: bool,
}

impl Common {
    fn map(&self) -> Result<MapFile> {
        Ok(match (&self.map, self.empty) {
            (Some(p), _) => MapFile::load(p)?,
            (None, true) => MapFile::default(),
            (None, false) => MapFile::perth(),
        })
    }

    fn plan(&self) -> Result<(Plan, MapFile)> {
        let (_, services) = tvmap::load(&self.zip)?;
        let map = self.map()?;
        Ok((Plan::build(services, &map), map))
    }
}

/// The TV looks for the original file name on the stick, so keep it and put
/// the result in its own folder.
fn default_output(zip: &Path) -> PathBuf {
    let name = zip.file_name().unwrap_or_default();
    PathBuf::from("tvmap-out").join(name)
}

fn ensure_parent(p: &Path) -> Result<()> {
    if let Some(dir) = p.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}

fn flags(hidden: bool, skip: bool) -> &'static str {
    match (hidden, skip) {
        (true, true) => "hidden,skip",
        (true, false) => "hidden",
        (false, true) => "skip",
        (false, false) => "",
    }
}

fn print_row(r: &Row) {
    let t = r.target();
    let arrow = if r.is_changed() { "->" } else { "  " };
    let key = r.svc.key.to_string();
    println!(
        "{:>5} {arrow} {:<5} {:<5} {:<24} {:<16} {:<11} {}",
        r.svc.major,
        t.number,
        r.svc.kind.label(),
        r.svc.name,
        key,
        flags(t.hidden, t.skip),
        r.status.describe(),
    );
}

fn print_plan(plan: &Plan) {
    println!(
        "{:>5}    {:<5} {:<5} {:<24} {:<16} {:<11} status",
        "now", "new", "type", "name", "onid/tsid/sid", "flags"
    );
    for r in &plan.rows {
        print_row(r);
    }
    let changed = plan.rows.iter().filter(|r| r.is_changed()).count();
    let attention = plan
        .rows
        .iter()
        .filter(|r| r.status.needs_attention())
        .count();
    println!(
        "\n{} services, {changed} to change, {attention} need a look",
        plan.rows.len()
    );
    for (n, rows) in plan.conflicts() {
        let names: Vec<&str> = rows
            .iter()
            .map(|&i| plan.rows[i].svc.name.as_str())
            .collect();
        println!("CONFLICT: {n} used by {}", names.join(", "));
    }
}

fn main() -> std::process::ExitCode {
    match run(Cli::parse()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.cmd {
        Cmd::List(c) => print_plan(&c.plan()?.0),
        Cmd::Apply {
            common,
            output,
            dry_run,
            save_map,
        } => {
            let (plan, map) = common.plan()?;
            print_plan(&plan);
            if dry_run {
                return Ok(());
            }
            if !plan.conflicts().is_empty() {
                bail!("fix the conflicts above first (try `tvmap edit`)");
            }
            let out = output.unwrap_or_else(|| default_output(&common.zip));
            ensure_parent(&out)?;
            let n = tvmap::save(&common.zip, &plan, &out)?;
            println!("\nwrote {} ({n} services changed)", out.display());
            if let Some(p) = save_map {
                ensure_parent(&p)?;
                plan.to_map(map.region).save(&p)?;
                println!("saved map to {}", p.display());
            }
        }
        Cmd::Edit {
            common,
            output,
            save_map,
        } => {
            let (plan, map) = common.plan()?;
            let out = output.unwrap_or_else(|| default_output(&common.zip));
            let map_out = save_map.unwrap_or_else(|| PathBuf::from("tvmap-out/channel-map.toml"));
            tui::run(tui::Session {
                zip: common.zip,
                out,
                map_out,
                region: map.region,
                plan,
            })?;
        }
    }
    Ok(())
}
