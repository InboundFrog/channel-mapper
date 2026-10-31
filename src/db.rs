//! Reading and writing services in a Samsung channel database.
//!
//! Schema notes (firmware T-KTMDEUC-1410.1, see also ChanSort's Samsung loader):
//! - `SRV.major` is the channel number the TV shows.
//! - `SRV.tsid` + `SRV.progNum` + `SRV_DVB.onid` identify a broadcast service.
//! - `SRV_DVB.lcn` is the broadcaster's number. UK firmware leaves it at 65535
//!   for Australian broadcasts, which is why everything lands in the 800s.
//! - Names are TEXT holding UTF-16 with the byte order swapped. We read them
//!   as BLOBs and decode UTF-16BE. We never write them: writing TEXT back
//!   through SQLite re-encodes it and corrupts the list.

use anyhow::Result;
use rusqlite::{params, Connection};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ServiceKey {
    pub onid: i64,
    pub tsid: i64,
    pub sid: i64,
}

impl std::fmt::Display for ServiceKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}/{}", self.onid, self.tsid, self.sid)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Tv,
    TvHd,
    Radio,
    Data,
    Other(i64),
}

impl Kind {
    /// DVB service_type values as Samsung stores them.
    fn from_srv_type(t: i64) -> Self {
        match t {
            0x01 | 0x16 => Kind::Tv,
            0x11 | 0x19 | 0x1f => Kind::TvHd,
            0x02 | 0x0a => Kind::Radio,
            0x0c => Kind::Data,
            n => Kind::Other(n),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Kind::Tv => "TV",
            Kind::TvHd => "HD",
            Kind::Radio => "Radio",
            Kind::Data => "Data",
            Kind::Other(_) => "?",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Service {
    /// Which database entry in the zip this came from (e.g. `dvbt`).
    pub db: String,
    pub srv_id: i64,
    pub key: ServiceKey,
    pub name: String,
    pub kind: Kind,
    pub major: i64,
    pub lcn: i64,
    pub hidden: bool,
    pub skip: bool,
}

/// The settings we are allowed to change on a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    pub number: i64,
    pub hidden: bool,
    pub skip: bool,
}

impl Service {
    pub fn current(&self) -> Settings {
        Settings {
            number: self.major,
            hidden: self.hidden,
            skip: self.skip,
        }
    }
}

/// Decode Samsung's byte-swapped UTF-16 name.
pub fn decode_name(raw: &[u8]) -> String {
    let units: Vec<u16> = raw
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_be_bytes(*c))
        .collect();
    String::from_utf16_lossy(&units)
        .trim_end_matches('\0')
        .to_string()
}

pub fn read_services(conn: &Connection, db: &str) -> Result<Vec<Service>> {
    let mut stmt = conn.prepare(
        "select s.srvId, d.onid, s.tsid, s.progNum, cast(s.srvName as blob), s.srvType,
                s.major, d.lcn, s.hidden, s.numSel
           from SRV s join SRV_DVB d using(srvId)
          where ifnull(s.elim, 0) = 0
          order by s.major",
    )?;
    let rows = stmt.query_map([], |r| {
        let raw: Option<Vec<u8>> = r.get(4)?;
        Ok(Service {
            db: db.to_string(),
            srv_id: r.get(0)?,
            key: ServiceKey {
                onid: r.get(1)?,
                tsid: r.get(2)?,
                sid: r.get(3)?,
            },
            name: raw.as_deref().map(decode_name).unwrap_or_default(),
            kind: Kind::from_srv_type(r.get::<_, Option<i64>>(5)?.unwrap_or(-1)),
            major: r.get(6)?,
            lcn: r.get::<_, Option<i64>>(7)?.unwrap_or(65535),
            hidden: r.get::<_, Option<bool>>(8)?.unwrap_or(false),
            // numSel = "selectable by CH+/-"; skip is its inverse.
            skip: !r.get::<_, Option<bool>>(9)?.unwrap_or(true),
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub struct Change {
    pub srv_id: i64,
    pub old: Settings,
    pub new: Settings,
}

/// Apply changes in one transaction. Only the columns for settings that
/// actually differ are written: `major`; `hidden` + `hideGuide`; `numSel`.
pub fn write_settings(conn: &mut Connection, changes: &[Change]) -> Result<usize> {
    let tx = conn.transaction()?;
    let mut n = 0;
    {
        let mut set_major = tx.prepare("update SRV set major = ?1 where srvId = ?2")?;
        let mut set_hidden =
            tx.prepare("update SRV set hidden = ?1, hideGuide = ?1 where srvId = ?2")?;
        let mut set_skip = tx.prepare("update SRV set numSel = ?1 where srvId = ?2")?;
        for c in changes {
            let mut touched = false;
            if c.old.number != c.new.number {
                set_major.execute(params![c.new.number, c.srv_id])?;
                touched = true;
            }
            if c.old.hidden != c.new.hidden {
                set_hidden.execute(params![c.new.hidden, c.srv_id])?;
                touched = true;
            }
            if c.old.skip != c.new.skip {
                set_skip.execute(params![!c.new.skip, c.srv_id])?;
                touched = true;
            }
            n += usize::from(touched);
        }
    }
    tx.commit()?;
    // Samsung's own files sometimes have stale indexes; ChanSort does this too.
    conn.execute_batch("REINDEX")?;
    Ok(n)
}
