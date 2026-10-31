use tvmap::db::decode_name;
use tvmap::mapfile::MapFile;
use tvmap::plan::{names_similar, normalise};

#[test]
fn decodes_byte_swapped_utf16() {
    // "7two" as UTF-16BE, with a trailing NUL.
    let raw = [0, b'7', 0, b't', 0, b'w', 0, b'o', 0, 0];
    assert_eq!(decode_name(&raw), "7two");
}

#[test]
fn normalises_names() {
    assert_eq!(normalise("7two Perth"), "7two");
    assert_eq!(normalise("9Go!HD Perth"), "9gohd");
    assert_eq!(normalise("ABC Kids/ABC Family"), "abckidsabcfamily");
    assert_eq!(normalise("7two Sydney"), "7two");
    assert_eq!(normalise("Channel 9 Darwin"), "channel9");
}

#[test]
fn similar_names() {
    assert!(names_similar("7two Perth", "7two"));
    assert!(names_similar("ABC TV", "ABCTV HD"));
    assert!(names_similar("9Gem HD Perth", "9GemHD Perth"));
    assert!(names_similar("10 Peach", "10 peach"));
    assert!(!names_similar("TVSN", "Seven News 24"));
    assert!(!names_similar("10 comedy", "Nickelodeon"));
}

#[test]
fn perth_map_is_valid() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/maps/au/wa/perth.toml");
    let m = MapFile::load(std::path::Path::new(path)).unwrap();
    assert_eq!(m.channels.len(), 58);
}

#[test]
fn bundled_maps_are_valid() {
    fn walk(dir: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(&p, found);
            } else if p.extension().is_some_and(|x| x == "toml") {
                found.push(p);
            }
        }
    }
    let mut maps = Vec::new();
    walk(
        std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/maps")),
        &mut maps,
    );
    assert_eq!(maps.len(), 8, "{maps:?}");
    for p in maps {
        let m = MapFile::load(&p).unwrap_or_else(|e| panic!("{e:#}"));
        assert!(m.region.is_some(), "{}", p.display());
        assert!(!m.channels.is_empty(), "{}", p.display());
    }
}

#[test]
fn rejects_bad_maps() {
    let dup = r#"
        [[channel]]
        onid = 1
        tsid = 2
        sid = 3
        name = "a"
        number = 7
        [[channel]]
        onid = 1
        tsid = 2
        sid = 3
        name = "b"
        number = 8
    "#;
    assert!(MapFile::parse(dup).is_err());
    let range = "[[channel]]\nonid = 1\ntsid = 2\nsid = 3\nname = \"a\"\nnumber = 0\n";
    assert!(MapFile::parse(range).is_err());
}
