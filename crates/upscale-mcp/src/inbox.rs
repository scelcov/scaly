use std::path::{Path, PathBuf};
use std::time::SystemTime;

use base64::Engine;
use upscale_core::Error;

const IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "bmp"];
const VIDEO_EXTS: &[&str] = &["mp4", "webm", "mkv", "mov", "m4v"];

/// Newest image LibreChat saved from a paste or upload.
///
/// # Errors
///
/// Returns [`Error::NoChatImage`] when the folder has no image.
pub fn newest_image(inbox: &Path) -> Result<PathBuf, Error> {
    let mut newest: Option<(SystemTime, PathBuf)> = None;
    visit(inbox, IMAGE_EXTS, &mut newest)?;
    newest.map(|(_, path)| path).ok_or(Error::NoChatImage)
}

/// Newest video LibreChat saved, from the image inbox or the sibling uploads folder.
///
/// # Errors
///
/// Returns [`Error::NoChatVideo`] when neither folder has a video.
pub fn newest_video(inbox: &Path) -> Result<PathBuf, Error> {
    let mut newest: Option<(SystemTime, PathBuf)> = None;
    visit(inbox, VIDEO_EXTS, &mut newest)?;
    if let Some(parent) = inbox.parent() {
        visit(&parent.join("uploads"), VIDEO_EXTS, &mut newest)?;
    }
    newest.map(|(_, path)| path).ok_or(Error::NoChatVideo)
}

/// Decode a raw base64 string or a `data:image/...;base64,` URL.
///
/// # Errors
///
/// Returns [`Error::Upscale`] when the payload is not valid base64.
pub fn decode_image(raw: &str) -> Result<(Vec<u8>, &'static str), Error> {
    let trimmed = raw.trim();
    let (mime, payload) = if let Some(rest) = trimmed.strip_prefix("data:") {
        let (meta, data) = rest
            .split_once(',')
            .ok_or_else(|| Error::Upscale("image data url has no payload".to_owned()))?;
        let mime = meta.split(';').next().unwrap_or("image/png");
        (mime_ext(mime), data)
    } else {
        ("png", trimmed)
    };
    let cleaned: String = payload.chars().filter(|ch| !ch.is_whitespace()).collect();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(cleaned.as_bytes())
        .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(cleaned.as_bytes()))
        .map_err(|_| Error::Upscale("image data is not valid base64".to_owned()))?;
    if bytes.is_empty() {
        return Err(Error::Upscale("image data is empty".to_owned()));
    }
    Ok((bytes, mime))
}

pub fn write_image(bytes: &[u8], ext: &str) -> Result<PathBuf, Error> {
    let dir = std::env::temp_dir().join("upscale-in");
    std::fs::create_dir_all(&dir)
        .map_err(|err| Error::Upscale(format!("failed to create input directory: {err}")))?;
    let path = dir.join(format!("{}.{}", upscale_core::JobId::new(), ext));
    std::fs::write(&path, bytes)
        .map_err(|err| Error::Upscale(format!("failed to write pasted image: {err}")))?;
    Ok(path)
}

pub fn default_output(kind_ext: &str) -> PathBuf {
    std::env::temp_dir().join("upscale-out").join(format!(
        "{}.{}",
        upscale_core::JobId::new(),
        kind_ext
    ))
}

fn visit(
    dir: &Path,
    extensions: &[&str],
    newest: &mut Option<(SystemTime, PathBuf)>,
) -> Result<(), Error> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(Error::Upscale(format!(
                "failed to read {}: {err}",
                dir.display()
            )));
        }
    };
    for entry in entries {
        let entry =
            entry.map_err(|err| Error::Upscale(format!("failed to read a chat file: {err}")))?;
        let path = entry.path();
        if path.is_dir() {
            visit(&path, extensions, newest)?;
            continue;
        }
        if !has_extension(&path, extensions) {
            continue;
        }
        let modified = entry
            .metadata()
            .and_then(|meta| meta.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        if newest.as_ref().is_none_or(|(time, _)| modified >= *time) {
            *newest = Some((modified, path));
        }
    }
    Ok(())
}

fn has_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| extensions.contains(&ext.to_ascii_lowercase().as_str()))
}

fn mime_ext(mime: &str) -> &'static str {
    match mime {
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        _ => "png",
    }
}
