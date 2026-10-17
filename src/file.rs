//! Local Markdown file loading, independent of the UI thread.

use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use mira::{
    code_layout::{self, CodeLayout},
    document::Document,
    markdown,
};

/// A parsed document and its resolved local origin.
pub struct LoadedDocument {
    /// Canonical path used to resolve document-relative resources.
    pub path: PathBuf,
    /// Immutable Markdown representation parsed once during loading.
    pub document: Document,
    /// File opening, validation, UTF-8 reading, and path resolution time.
    pub read_time: Duration,
    /// Markdown parsing and document-model construction time.
    pub parse_time: Duration,
    /// Source-backed line indexes for oversized code blocks.
    pub code: HashMap<usize, CodeLayout>,
    /// Code indexing time, kept separate from Markdown parsing.
    pub code_index_time: Duration,
}

/// Loads a regular UTF-8 `.md` or `.markdown` file without writing to it.
///
/// Call from a background executor. Returns an I/O error for unsupported
/// extensions, directories, missing/unreadable files, or invalid UTF-8.
pub fn load(path: &Path) -> io::Result<LoadedDocument> {
    let start = Instant::now();
    if !is_markdown(path) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Choose a .md or .markdown file.",
        ));
    }
    let mut file = File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The selected path is not a regular file.",
        ));
    }
    let mut source = String::new();
    file.read_to_string(&mut source)?;
    let path = path.canonicalize()?;
    let read_time = start.elapsed();
    let parse_start = Instant::now();
    let document = markdown::parse(source);
    let parse_time = parse_start.elapsed();
    let index_start = Instant::now();
    let code = code_layout::index(&document);
    let code_index_time = index_start.elapsed();
    Ok(LoadedDocument {
        path,
        document,
        read_time,
        parse_time,
        code,
        code_index_time,
    })
}

/// Whether a path has a supported Markdown extension, case-insensitively.
pub fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown")
        })
}
