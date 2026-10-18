//! Source-backed Markdown blocks, independent of the window and renderer.

use std::ops::Range;

use pulldown_cmark::Alignment;

/// One parsed document, retaining its original UTF-8 source exactly once.
#[derive(Debug)]
pub struct Document {
    /// Original Markdown, never modified by the viewer.
    pub source: String,
    /// Renderable leaves in document order.
    pub blocks: Vec<Block>,
    /// Shared ancestry for nested quotes and list items.
    pub contexts: Vec<BlockContext>,
    /// Link and image destinations, indexed by inline runs.
    pub targets: Vec<String>,
    /// Shared metadata for table rows.
    pub tables: Vec<Table>,
}

/// Display text either borrowed by byte range or normalized by the parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Text {
    /// A UTF-8 byte range in the document source.
    Source(Range<usize>),
    /// Text whose Markdown spelling differs from its display value.
    Literal(Box<str>),
}

impl Text {
    /// Returns display text using the source from which this value was parsed.
    pub fn as_str<'a>(&'a self, source: &'a str) -> &'a str {
        match self {
            Self::Source(range) => source.get(range.clone()).unwrap_or_default(),
            Self::Literal(text) => text,
        }
    }
}

/// Formatting combined across nested inline tags.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct InlineStyle {
    /// Strong emphasis.
    pub bold: bool,
    /// Emphasis.
    pub italic: bool,
    /// Strikethrough.
    pub strike: bool,
    /// Monospace inline code.
    pub code: bool,
}

/// A text run or image at its original inline position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inline {
    /// Original Markdown byte range, including syntax when applicable.
    pub source_range: Range<usize>,
    /// Display text, or plain alternative text for an image.
    pub text: Text,
    /// Combined text formatting.
    pub style: InlineStyle,
    /// Link destination index, including links around images.
    pub link: Option<usize>,
    /// Image destination index, if this run represents an image.
    pub image: Option<usize>,
}

/// A leaf suitable for a variable-height document list.
#[derive(Debug, PartialEq)]
pub struct Block {
    /// Original Markdown bytes associated with this block.
    pub source_range: Range<usize>,
    /// Immediate enclosing context index.
    pub context: Option<usize>,
    /// Whether this is the first leaf of its enclosing list item.
    pub marker: bool,
    /// Renderable content.
    pub kind: BlockKind,
}

/// Native-renderable Markdown content.
#[derive(Debug, PartialEq)]
pub enum BlockKind {
    /// A paragraph, including implicit tight-list paragraphs and literal raw HTML.
    Paragraph(Vec<Inline>),
    /// A heading with level from one through six.
    Heading {
        /// CommonMark heading level.
        level: u8,
        /// Heading text and formatting.
        runs: Vec<Inline>,
    },
    /// Fenced or indented code with parser-preserved whitespace.
    Code {
        /// Code text without Markdown delimiters.
        runs: Vec<Inline>,
        /// Fenced code information string, when present.
        language: Option<Text>,
    },
    /// A thematic horizontal rule.
    Rule,
    /// A paragraph consisting solely of one image.
    Image {
        /// Image destination index.
        target: usize,
        /// Image alternative text.
        alt: Vec<Inline>,
        /// Enclosing link destination index.
        link: Option<usize>,
    },
    /// One independently renderable table row.
    TableRow {
        /// Shared table metadata index.
        table: usize,
        /// Whether this is the header row.
        header: bool,
        /// Cell text and formatting in column order.
        cells: Vec<Vec<Inline>>,
    },
}

/// One shared ancestor in an iterative context arena.
#[derive(Debug, PartialEq, Eq)]
pub struct BlockContext {
    /// Enclosing context index.
    pub parent: Option<usize>,
    /// Quote or list item presentation.
    pub kind: ContextKind,
}

/// Structural context retained without nested renderer trees.
#[derive(Debug, PartialEq, Eq)]
pub enum ContextKind {
    /// A blockquote ancestor.
    Quote,
    /// A read-only ordered, unordered, or task-list item.
    ListItem {
        /// Item number, or `None` for unordered lists.
        number: Option<u64>,
        /// Checkbox state, or `None` for ordinary list items.
        checked: Option<bool>,
    },
}

/// Shared table column alignment.
#[derive(Debug, PartialEq)]
pub struct Table {
    /// Alignment for each declared column.
    pub alignments: Vec<Alignment>,
}
