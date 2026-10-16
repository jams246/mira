//! Link targets that cannot execute embedded content or arbitrary local files.

use std::path::{Path, PathBuf};

/// A supported destination activated explicitly by the reader.
#[derive(Debug, PartialEq, Eq)]
pub enum LinkTarget {
    /// HTTP, HTTPS, or mailto destination opened by the operating system.
    Web(String),
    /// Another local Markdown document, optionally at a heading.
    Document {
        /// Path relative to the current document's directory.
        path: PathBuf,
        /// Decoded heading fragment.
        anchor: Option<String>,
    },
    /// Heading within the current document; an empty fragment means the top.
    Anchor(String),
}

/// Resolves a supported link without file I/O or invoking the operating system.
///
/// Rejects executable/custom schemes, absolute local paths, control characters,
/// malformed percent encodings, and local targets without a Markdown extension.
pub fn resolve(document_path: &Path, target: &str) -> Result<LinkTarget, String> {
    if target.chars().any(char::is_control) {
        return Err("Link contains control characters.".into());
    }
    if let Some(anchor) = target.strip_prefix('#') {
        return decode(anchor).map(LinkTarget::Anchor);
    }
    if target.is_empty() {
        return Ok(LinkTarget::Anchor(String::new()));
    }
    if let Some((scheme, rest)) = target.split_once(':') {
        let scheme = scheme.to_ascii_lowercase();
        match scheme.as_str() {
            "http" | "https" => {
                let authority = rest
                    .strip_prefix("//")
                    .and_then(|rest| rest.split(['/', '?', '#']).next());
                if authority.is_none_or(str::is_empty)
                    || target.chars().any(char::is_whitespace)
                    || target.contains('\\')
                {
                    return Err("Invalid web link.".into());
                }
            }
            "mailto" if !rest.is_empty() => {}
            _ => return Err(format!("Unsupported link scheme: {scheme}.")),
        }
        return Ok(LinkTarget::Web(target.into()));
    }
    let (relative, anchor) = target
        .split_once('#')
        .map_or((target, None), |(path, anchor)| (path, Some(anchor)));
    let relative = decode(relative)?;
    if relative.starts_with(['/', '\\']) || relative.contains(':') {
        return Err("Only relative local Markdown links are supported.".into());
    }
    let path = Path::new(&relative);
    if !path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown")
        })
    {
        return Err("Local links must target a .md or .markdown file.".into());
    }
    Ok(LinkTarget::Document {
        path: document_path.parent().unwrap_or(Path::new(".")).join(path),
        anchor: anchor.map(decode).transpose()?,
    })
}

/// Decodes percent-escaped UTF-8 without treating a literal plus as a space.
pub fn decode(value: &str) -> Result<String, String> {
    let mut decoded = Vec::with_capacity(value.len());
    let mut bytes = value.bytes();
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let high = bytes.next().and_then(|byte| (byte as char).to_digit(16));
            let low = bytes.next().and_then(|byte| (byte as char).to_digit(16));
            let (Some(high), Some(low)) = (high, low) else {
                return Err("Invalid percent encoding in link.".into());
            };
            decoded.push((high * 16 + low) as u8);
        } else {
            decoded.push(byte);
        }
    }
    let decoded = String::from_utf8(decoded).map_err(|_| "Link is not valid UTF-8.".to_string())?;
    if decoded.chars().any(char::is_control) {
        return Err("Link contains control characters.".into());
    }
    Ok(decoded)
}

/// Builds a heading fragment using readable Unicode letters and numbers.
pub fn heading_slug(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter_map(|character| {
            if character.is_whitespace() {
                Some('-')
            } else if character.is_alphanumeric() || matches!(character, '-' | '_') {
                Some(character)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
