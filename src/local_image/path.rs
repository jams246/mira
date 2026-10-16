use std::fs::{self, File};
use std::path::{Component, Path};

pub(super) fn open(document: &Path, target: &str) -> Result<File, String> {
    let target = crate::links::decode(target)?;
    if target.is_empty() || target.starts_with(['/', '\\']) || target.contains(':') {
        return Err("Only relative local images are supported.".into());
    }
    let relative = Path::new(&target);
    let mut depth = 0usize;
    for component in relative.components() {
        match component {
            Component::Normal(_) => depth += 1,
            Component::CurDir => {}
            Component::ParentDir if depth > 0 => depth -= 1,
            _ => return Err("Image path escapes the Markdown directory.".into()),
        }
    }
    let parent = document
        .parent()
        .ok_or("Markdown file has no parent directory.")?;
    let parent = fs::canonicalize(parent)
        .map_err(|error| format!("Cannot resolve Markdown directory: {error}"))?;
    let image = fs::canonicalize(parent.join(relative))
        .map_err(|error| format!("Cannot find image: {error}"))?;
    if !image.starts_with(&parent) {
        return Err("Image path escapes the Markdown directory.".into());
    }
    let file = File::open(&image).map_err(|error| format!("Cannot open image: {error}"))?;
    #[cfg(windows)]
    if !final_path(&file)?.starts_with(&parent) {
        return Err("Image path escapes the Markdown directory.".into());
    }
    if !file
        .metadata()
        .map_err(|error| format!("Cannot inspect image: {error}"))?
        .is_file()
    {
        return Err("Image path does not identify a regular file.".into());
    }
    Ok(file)
}

#[cfg(windows)]
fn final_path(file: &File) -> Result<std::path::PathBuf, String> {
    use std::os::windows::ffi::OsStringExt;
    use std::os::windows::io::AsRawHandle;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFinalPathNameByHandleW(
            handle: *mut std::ffi::c_void,
            path: *mut u16,
            length: u32,
            flags: u32,
        ) -> u32;
    }

    // SAFETY: The borrowed file keeps its handle valid; null requests the path length.
    let needed =
        unsafe { GetFinalPathNameByHandleW(file.as_raw_handle(), std::ptr::null_mut(), 0, 0) };
    if needed > 32768 {
        return Err("Image path exceeds the Windows path limit.".into());
    }
    if needed == 0 {
        return Err(format!(
            "Cannot verify image path: {}",
            std::io::Error::last_os_error()
        ));
    }
    let mut path = vec![0u16; needed as usize + 1];
    // SAFETY: The live handle and writable buffer are valid for the supplied length.
    let length = unsafe {
        GetFinalPathNameByHandleW(
            file.as_raw_handle(),
            path.as_mut_ptr(),
            path.len() as u32,
            0,
        )
    };
    if length == 0 || length as usize >= path.len() {
        return Err(format!(
            "Cannot verify image path: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(std::ffi::OsString::from_wide(&path[..length as usize]).into())
}
