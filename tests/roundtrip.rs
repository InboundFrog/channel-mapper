//! End-to-end tests against a real export from a UE40MU6400 (firmware
//! T-KTMDEUC-1410.1) tuned in Perth.

use rusqlite::Connection;
use std::io::Read;
use std::path::{Path, PathBuf};
use tvmap::mapfile::MapFile;
use tvmap::plan::{Plan, Status};

const FIXTURE: &str = "tests/fixtures/Channel_list_T-KTMDEUC-1410.1.zip";

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE)
}

fn perth_map() -> MapFile {
    MapFile::load(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/maps/au/wa/perth.toml"
    )))
    .unwrap()
}

fn perth_plan() -> Plan {
    let (_, services) = tvmap::load(&fixture()).unwrap();
    Plan::build(services, &perth_map())
}

fn row(plan: &Plan, sid: i64) -> &tvmap::plan::Row {
    plan.rows.iter().find(|r| r.svc.key.sid == sid).unwrap()
}

/// Extract one zip entry to a temp file and open it.
fn open_entry(zip: &Path, name: &str) -> (tempfile::NamedTempFile, Connection) {
    let mut z = zip::ZipArchive::new(std::fs::File::open(zip).unwrap()).unwrap();
    let mut buf = Vec::new();
    z.by_name(name).unwrap().read_to_end(&mut buf).unwrap();
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), &buf).unwrap();
    let conn = Connection::open(tmp.path()).unwrap();
    (tmp, conn)
}

/// Every table, every row, every column, as raw bytes: (table, column, rowid, value).
fn dump(conn: &Connection) -> Vec<(String, String, i64, Option<Vec<u8>>)> {
    let tables: Vec<String> = conn
        .prepare("select name from sqlite_master where type='table' order by name")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let mut out = Vec::new();
    for t in tables {
        let cols: Vec<String> = conn
            .prepare(&format!("pragma table_info({t})"))
            .unwrap()
            .query_map([], |r| r.get(1))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        for c in cols {
            let mut stmt = conn
                .prepare(&format!(
                    "select rowid, cast({c} as blob) from {t} order by rowid"
                ))
                .unwrap();
            let rows = stmt
                .query_map([], |r| {
                    Ok((r.get::<_, i64>(0)?, r.get::<_, Option<Vec<u8>>>(1)?))
                })
                .unwrap();
            for r in rows {
                let (id, v) = r.unwrap();
                out.push((t.clone(), c.clone(), id, v));
            }
        }
    }
    out
}

fn diff_columns(a: &Path, b: &Path) -> Vec<(String, String)> {
    let (_ta, ca) = open_entry(a, "dvbt");
    let (_tb, cb) = open_entry(b, "dvbt");
    let (da, db) = (dump(&ca), dump(&cb));
    assert_eq!(da.len(), db.len(), "row/column count changed");
    let mut cols: Vec<(String, String)> = da
        .iter()
        .zip(&db)
        .filter(|(x, y)| x != y)
        .map(|(x, _)| (x.0.clone(), x.1.clone()))
        .collect();
    cols.sort();
    cols.dedup();
    cols
}

#[test]
fn reads_the_export() {
    let (archive, services) = tvmap::load(&fixture()).unwrap();
    assert_eq!(archive.dbs.len(), 1);
    assert_eq!(archive.dbs[0].entry, "dvbt");
    assert_eq!(services.len(), 58);
    assert!(services.iter().all(|s| (800..=857).contains(&s.major)));
    // The UK firmware dropped the Australian LCNs.
    assert!(services.iter().all(|s| s.lcn == 65535));
    let names: Vec<&str> = services.iter().map(|s| s.name.as_str()).collect();
    for n in [
        "7 Perth",
        "9HD Perth",
        "ABC TV",
        "SBS ONE HD",
        "10 HD",
        "triple j",
    ] {
        assert!(names.contains(&n), "missing {n}: {names:?}");
    }
}

#[test]
fn perth_map_covers_everything() {
    let plan = perth_plan();
    let odd: Vec<_> = plan
        .rows
        .iter()
        .filter(|r| r.status != Status::Matched)
        .map(|r| (&r.svc.name, &r.status))
        .collect();
    assert!(odd.is_empty(), "{odd:?}");
    assert!(plan.conflicts().is_empty());
    assert_eq!(row(&plan, 1376).target().number, 7);
    assert_eq!(row(&plan, 1378).target().number, 72);
    assert_eq!(row(&plan, 1122).target().number, 9);
    assert_eq!(row(&plan, 741).target().number, 2); // ABCTV HD
    assert_eq!(row(&plan, 737).target().number, 20); // ABC TV (SD)
    assert_eq!(row(&plan, 805).target().number, 3); // SBS ONE HD
    assert_eq!(row(&plan, 801).target().number, 30); // SBS ONE (SD)
    assert_eq!(row(&plan, 1665).target().number, 10);
}

#[test]
fn apply_changes_only_channel_numbers() {
    let plan = perth_plan();
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.zip");
    assert_eq!(tvmap::save(&fixture(), &plan, &out).unwrap(), 58);

    // Only SRV.major differs; names etc. are byte-identical.
    assert_eq!(
        diff_columns(&fixture(), &out),
        vec![("SRV".into(), "major".into())]
    );

    // Re-reading gives the planned numbers.
    let (_, services) = tvmap::load(&out).unwrap();
    for s in &services {
        let want = plan
            .rows
            .iter()
            .find(|r| r.svc.srv_id == s.srv_id)
            .unwrap()
            .target();
        assert_eq!(s.major, want.number, "{}", s.name);
    }

    let (_t, conn) = open_entry(&out, "dvbt");
    let ok: String = conn
        .query_row("pragma integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(ok, "ok");
    let types: Vec<String> = conn
        .prepare("select distinct typeof(srvName) from SRV")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(types, vec!["text"]);
}

#[test]
fn other_zip_entries_are_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.zip");
    tvmap::save(&fixture(), &perth_plan(), &out).unwrap();

    let mut a = zip::ZipArchive::new(std::fs::File::open(fixture()).unwrap()).unwrap();
    let mut b = zip::ZipArchive::new(std::fs::File::open(&out).unwrap()).unwrap();
    assert_eq!(a.len(), b.len());
    for i in 0..a.len() {
        let mut ea = a.by_index_raw(i).unwrap();
        let name = ea.name().to_string();
        let mut eb = b.by_index_raw(i).unwrap();
        assert_eq!(name, eb.name(), "entry order changed");
        if name == "dvbt" {
            continue;
        }
        let (mut x, mut y) = (Vec::new(), Vec::new());
        ea.read_to_end(&mut x).unwrap();
        eb.read_to_end(&mut y).unwrap();
        assert_eq!(x, y, "{name} changed");
    }
}

#[test]
fn hide_and_skip_touch_only_their_columns() {
    let mut plan = perth_plan();
    for r in &mut plan.rows {
        r.number = None; // keep every number
    }
    let tvsn = plan
        .rows
        .iter_mut()
        .find(|r| r.svc.key.sid == 1385)
        .unwrap();
    tvsn.hidden = true;
    let radio = plan.rows.iter_mut().find(|r| r.svc.key.sid == 745).unwrap();
    radio.skip = true;

    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.zip");
    assert_eq!(tvmap::save(&fixture(), &plan, &out).unwrap(), 2);
    assert_eq!(
        diff_columns(&fixture(), &out),
        vec![
            ("SRV".into(), "hidden".into()),
            ("SRV".into(), "hideGuide".into()),
            ("SRV".into(), "numSel".into()),
        ]
    );
    let (_, services) = tvmap::load(&out).unwrap();
    let get = |sid| services.iter().find(|s| s.key.sid == sid).unwrap();
    assert!(get(1385).hidden && !get(1385).skip);
    assert!(get(745).skip && !get(745).hidden);
}

#[test]
fn refuses_duplicate_numbers() {
    let mut plan = perth_plan();
    plan.rows
        .iter_mut()
        .find(|r| r.svc.key.sid == 1385)
        .unwrap()
        .number = Some(7);
    assert_eq!(plan.conflicts().get(&7).map(Vec::len), Some(2));
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.zip");
    let err = tvmap::save(&fixture(), &plan, &out)
        .unwrap_err()
        .to_string();
    assert!(err.contains("duplicate"), "{err}");
    assert!(!out.exists());
}

#[test]
fn refuses_to_overwrite_the_original() {
    let dir = tempfile::tempdir().unwrap();
    let copy = dir.path().join("in.zip");
    std::fs::copy(fixture(), &copy).unwrap();
    let before = std::fs::read(&copy).unwrap();
    assert!(tvmap::save(&copy, &perth_plan(), &copy).is_err());
    assert_eq!(std::fs::read(&copy).unwrap(), before);
}

#[test]
fn saved_map_reproduces_the_plan() {
    let mut plan = perth_plan();
    // A manual tweak and a kept number should survive the round trip.
    plan.rows
        .iter_mut()
        .find(|r| r.svc.key.sid == 1385)
        .unwrap()
        .number = None;
    plan.rows
        .iter_mut()
        .find(|r| r.svc.key.sid == 745)
        .unwrap()
        .skip = true;

    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("map.toml");
    plan.to_map(Some("Perth".into())).save(&p).unwrap();
    let again = Plan::build(
        tvmap::load(&fixture()).unwrap().1,
        &MapFile::load(&p).unwrap(),
    );
    for r in &plan.rows {
        let r2 = again
            .rows
            .iter()
            .find(|x| x.svc.srv_id == r.svc.srv_id)
            .unwrap();
        assert_eq!(r.target(), r2.target(), "{}", r.svc.name);
        assert_eq!(r2.status, Status::Matched);
    }
}

#[test]
fn flags_renamed_services() {
    let mut map = perth_map();
    map.channels
        .iter_mut()
        .find(|e| e.sid == 1385)
        .unwrap()
        .name = "Seven News 24".into();
    let plan = Plan::build(tvmap::load(&fixture()).unwrap().1, &map);
    let r = row(&plan, 1385);
    assert_eq!(
        r.status,
        Status::NameChanged {
            expected: "Seven News 24".into()
        }
    );
    // Still proposes the mapped number; it's just flagged.
    assert_eq!(r.target().number, 77);
}

#[test]
fn falls_back_to_name_when_service_id_changes() {
    let mut map = perth_map();
    map.channels.iter_mut().find(|e| e.sid == 1378).unwrap().sid = 9999; // 7two
    let plan = Plan::build(tvmap::load(&fixture()).unwrap().1, &map);
    let r = row(&plan, 1378);
    assert!(
        matches!(r.status, Status::MatchedByName { .. }),
        "{:?}",
        r.status
    );
    assert_eq!(r.target().number, 72);
}

#[test]
fn ambiguous_name_fallback_is_left_unmatched() {
    // Three "7 Perth" services; if all their IDs changed we can't tell them apart.
    let mut map = perth_map();
    for e in map.channels.iter_mut().filter(|e| e.name == "7 Perth") {
        e.sid += 10_000;
    }
    let plan = Plan::build(tvmap::load(&fixture()).unwrap().1, &map);
    for sid in [1376, 1377, 1380] {
        let r = row(&plan, sid);
        assert_eq!(r.status, Status::Unmatched);
        assert_eq!(r.target().number, r.svc.major);
    }
}

#[test]
fn empty_map_changes_nothing() {
    let plan = Plan::build(tvmap::load(&fixture()).unwrap().1, &MapFile::default());
    assert!(plan
        .rows
        .iter()
        .all(|r| r.status == Status::Unmatched && !r.is_changed()));
}

#[test]
fn main_numbers_are_hd() {
    // The numbers people actually type should land on the HD copy.
    let plan = perth_plan();
    for n in [2, 3, 7, 9, 10] {
        let r = plan.rows.iter().find(|r| r.target().number == n).unwrap();
        assert_eq!(
            r.svc.kind,
            tvmap::db::Kind::TvHd,
            "{n} is {} ({:?})",
            r.svc.name,
            r.svc.kind
        );
    }
}
