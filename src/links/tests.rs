use super::*;

#[test]
fn resolves_unicode_relative_markdown_and_anchor() {
    assert_eq!(
        resolve(
            Path::new("docs/source.md"),
            "caf%C3%A9.markdown#hello%20world"
        ),
        Ok(LinkTarget::Document {
            path: PathBuf::from("docs/café.markdown"),
            anchor: Some("hello world".into()),
        })
    );
}

#[test]
fn resolves_unicode_heading_without_file_io() {
    assert_eq!(
        resolve(Path::new("source.md"), "#%E6%97%A5%E6%9C%AC%E8%AA%9E"),
        Ok(LinkTarget::Anchor("日本語".into()))
    );
}

#[test]
fn rejects_executable_and_custom_schemes() {
    for target in [
        "javascript:alert(1)",
        "file:///C:/Windows/calc.exe",
        "cmd:calc",
        "data:text/html,hi",
    ] {
        assert!(resolve(Path::new("source.md"), target).is_err(), "{target}");
    }
}

#[test]
fn rejects_executable_local_files_and_absolute_paths() {
    for target in [
        "calc.exe",
        "script.ps1",
        "//host/share.md",
        "/private.md",
        "%2Fprivate.md",
        "C%3A/private.md",
    ] {
        assert!(resolve(Path::new("source.md"), target).is_err(), "{target}");
    }
}

#[test]
fn rejects_invalid_encoding_and_encoded_control_characters() {
    for target in ["bad%XX.md", "bad%.md", "%FF.md", "%00.md", "#%0A"] {
        assert!(resolve(Path::new("source.md"), target).is_err(), "{target}");
    }
}

#[test]
fn accepts_supported_external_destinations() {
    for target in [
        "https://example.com/path",
        "HTTP://example.com",
        "mailto:reader@example.com",
    ] {
        assert_eq!(
            resolve(Path::new("source.md"), target),
            Ok(LinkTarget::Web(target.into()))
        );
    }
}

#[test]
fn rejects_malformed_web_links() {
    for target in [
        "https:",
        "https:///path",
        "http://",
        "https://example.com\\bad",
        "https://example.com/ space",
    ] {
        assert!(resolve(Path::new("source.md"), target).is_err(), "{target}");
    }
}

#[test]
fn preserves_plus_in_local_filenames() {
    assert_eq!(decode("C%2B%2B+Rust.md"), Ok("C+++Rust.md".into()));
}

#[test]
fn heading_slugs_preserve_unicode_and_strip_formatting_punctuation() {
    assert_eq!(heading_slug("Hello, 日本語 & café!"), "hello-日本語--café");
}
