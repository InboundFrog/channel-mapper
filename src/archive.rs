//! Reading and re-packing the Samsung channel-list `.zip`.
//!
//! The export holds several extension-less SQLite databases (`dvbt`, `sat`,
//! `ipsrv`, ...) plus `metadata.xml`. We only ever rewrite the broadcast
//! database(s); every other entry is copied across as raw compressed bytes so
//! it stays byte-identical.

use anyhow::{bail, Context, Result};
use rusqlite::{Connection, OpenFlags};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// One broadcast database pulled out of the archive into a temp file.
pub struct ChannelDb {
    /// Entry name inside the zip, e.g. `dvbt`.
    pub entry: String,
    file: NamedTempFile,
}

impl ChannelDb {
    pub fn path(&self) -> &Path {
        self.file.path()
    }

    pub fn open_ro(&self) -> Result<Connection> {
        Ok(Connection::open_with_flags(
            self.path(),
            OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?)
    }

    pub fn open_rw(&self) -> Result<Connection> {
        Ok(Connection::open_with_flags(
            self.path(),
            OpenFlags::SQLITE_OPEN_READ_WRITE,
        )?)
    }
}

pub struct ChannelArchive {
    pub source: PathBuf,
    pub dbs: Vec<ChannelDb>,
}

/// True if this SQLite file is a broadcast (DVB) channel database.
fn is_dvb_db(path: &Path) -> bool {
    let Ok(conn) = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY) else {
        return false;
    };
    let count: rusqlite::Result<i64> = conn.query_row(
        "select count(*) from sqlite_master where type='table' and name in ('SRV','SRV_DVB','CHNL')",
        [],
        |r| r.get(0),
    );
    matches!(count, Ok(3))
}

impl ChannelArchive {
    pub fn open(path: &Path) -> Result<Self> {
        let f = File::open(path).with_context(|| format!("opening {}", path.display()))?;
        let mut zip = ZipArchive::new(f).context("not a zip file")?;
        let mut dbs = Vec::new();
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i)?;
            let name = entry.name().to_string();
            // Samsung's databases have no extension and live at the top level.
            if name.contains('.') || name.contains('/') || name.contains('\\') || entry.is_dir() {
                continue;
            }
            let mut tmp = NamedTempFile::new()?;
            let mut buf = Vec::with_capacity(entry.size() as usize);
            entry.read_to_end(&mut buf)?;
            tmp.write_all(&buf)?;
            tmp.flush()?;
            if is_dvb_db(tmp.path()) {
                dbs.push(ChannelDb {
                    entry: name,
                    file: tmp,
                });
            }
        }
        if dbs.is_empty() {
            bail!(
                "{} has no broadcast channel database (expected an entry like `dvbt`)",
                path.display()
            );
        }
        Ok(Self {
            source: path.to_path_buf(),
            dbs,
        })
    }

    /// Write a new archive: rewritten databases are deflated fresh, every
    /// other entry is raw-copied. Entry order is preserved.
    pub fn save(&self, out: &Path) -> Result<()> {
        if same_file(&self.source, out) {
            bail!("refusing to overwrite the original export; pick a different output path");
        }
        let src = File::open(&self.source)?;
        let mut zin = ZipArchive::new(src)?;

        // Write to a temp file next to the output, then rename, so a failure
        // never leaves a half-written zip on the USB stick.
        let dir = out
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let tmp = NamedTempFile::new_in(dir)?;
        {
            let mut zout = ZipWriter::new(tmp.as_file());
            for i in 0..zin.len() {
                let name = zin.by_index_raw(i)?.name().to_string();
                if let Some(db) = self.dbs.iter().find(|d| d.entry == name) {
                    let bytes = std::fs::read(db.path())?;
                    let opts = SimpleFileOptions::default()
                        .compression_method(CompressionMethod::Deflated);
                    zout.start_file(name, opts)?;
                    zout.write_all(&bytes)?;
                } else {
                    zout.raw_copy_file(zin.by_index_raw(i)?)?;
                }
            }
            zout.finish()?;
        }
        tmp.persist(out)
            .with_context(|| format!("writing {}", out.display()))?;
        Ok(())
    }
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(x), Ok(y)) => x == y,
        _ => false,
    }
}
