use std::{fs, io::ErrorKind, path::PathBuf};

#[path = "../src/file.rs"]
mod file;

fn fixture(name: &str, bytes: &[u8]) -> PathBuf {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".verification/loading-tests");
    fs::create_dir_all(&directory).unwrap();
    let path = directory.join(name);
    fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn load_preserves_original_unicode_source_and_path() {
    let source = "# 你好\r\n\r\n**café** 😀\r\n";
    let path = fixture("Unicode filename 日本語 café.markdown", source.as_bytes());
    let loaded = file::load(&path).unwrap();
    eprintln!(
        "read {:?}, parse/model {:?}",
        loaded.read_time, loaded.parse_time
    );
    assert_eq!(loaded.document.source, source);
    assert_eq!(loaded.path, path.canonicalize().unwrap());
    assert_eq!(fs::read(path).unwrap(), source.as_bytes());
}

#[test]
fn load_accepts_empty_and_uppercase_markdown_extension() {
    let path = fixture("empty.MD", b"");
    assert!(file::load(&path).unwrap().document.blocks.is_empty());
}

#[test]
fn load_indexes_large_code_without_changing_source() {
    let line = "alpha\t日本語😀\n";
    let code = line.repeat(2000);
    let source = format!("```\nshort\n```\n\n```\n{code}```\n");
    let path = fixture("indexed-code.md", source.as_bytes());
    let loaded = file::load(&path).unwrap();
    eprintln!("code indexing {:?}", loaded.code_index_time);
    assert_eq!(loaded.code.len(), 1);
    let (&block, layout) = loaded.code.iter().next().unwrap();
    let mira::document::BlockKind::Code { runs, .. } = &loaded.document.blocks[block].kind else {
        panic!("indexed block must retain its code model");
    };
    assert_eq!(layout.line_count(), 2001);
    let first = layout.line(0).unwrap();
    assert_eq!(first.bytes, 0..line.len() - 1);
    assert!(!first.ascii);
    let slice = layout.slice(&loaded.document, runs, first.bytes);
    let text: String = slice
        .iter()
        .map(|run| run.text.as_str(&loaded.document.source))
        .collect();
    assert_eq!(text, line.trim_end_matches('\n'));
    assert_eq!(loaded.document.source, source);
    assert_eq!(fs::read(path).unwrap(), source.as_bytes());
}

#[test]
fn load_reports_missing_file() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".verification/loading-tests/never-created-missing.md");
    assert_eq!(file::load(&path).err().unwrap().kind(), ErrorKind::NotFound);
}

#[test]
fn load_rejects_invalid_utf8() {
    let path = fixture("invalid-utf8.md", &[0xff, 0xfe, 0x00]);
    assert_eq!(
        file::load(&path).err().unwrap().kind(),
        ErrorKind::InvalidData
    );
}

#[test]
fn load_rejects_unsupported_extension() {
    let path = fixture("unsupported.txt", b"plain");
    assert_eq!(
        file::load(&path).err().unwrap().kind(),
        ErrorKind::InvalidInput
    );
}

#[test]
fn load_supports_relative_paths() {
    let path = fixture("relative.md", b"plain text");
    let relative = path.strip_prefix(env!("CARGO_MANIFEST_DIR")).unwrap();
    assert_eq!(file::load(relative).unwrap().document.source, "plain text");
}

#[cfg(windows)]
#[test]
fn load_reports_unreadable_file_while_exclusively_locked() {
    use std::{fs::OpenOptions, os::windows::fs::OpenOptionsExt};
    let path = fixture("locked.md", b"locked fixture");
    let _lock = OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&path)
        .unwrap();
    assert!(file::load(&path).is_err());
}
