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
fn built_in_map_is_valid() {
    let m = MapFile::perth();
    assert_eq!(m.channels.len(), 58);
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
