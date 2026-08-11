/// Whitelist of video extensions. Episode match functions reject any
/// name not ending in one of these, so non-video files (subtitles,
/// images, fonts, ...) are never parsed as episodes.
pub(crate) const VIDEO_EXTS: &[&str] = &[".mkv", ".mp4", ".avi", ".m2ts"];

/// Whitelist of subtitle extensions. Subtitle files mirror video naming
/// (`Show S2E1.ass`), so low-confidence season rules may fire on them,
/// but never on unrelated files (images, fonts, notes, ...).
pub(crate) const SUBTITLE_EXTS: &[&str] = &[".ass", ".srt", ".ssa", ".sub", ".sup", ".idx", ".vtt"];

/// Whitelist of audio extensions (OSTs, drama CDs, character songs).
pub(crate) const AUDIO_EXTS: &[&str] = &[
    ".flac", ".mp3", ".wav", ".mka", ".ape", ".dts", ".tta", ".opus",
];

/// Whitelist of image extensions (scans, booklets, artwork).
pub(crate) const IMAGE_EXTS: &[&str] = &[
    ".jpg", ".jpeg", ".png", ".webp", ".bmp", ".gif", ".avif", ".cbz",
];

/// Whitelist of font extensions.
pub(crate) const FONT_EXTS: &[&str] = &[".ttf", ".ttc", ".otf"];

/// Whitelist of archive extensions (fonts / scans shipped as archives).
pub(crate) const ARCHIVE_EXTS: &[&str] = &[".rar", ".zip", ".7z"];

fn has_any_suffix(name: &str, exts: &[&str]) -> bool {
    let lower = name.to_lowercase();
    exts.iter().any(|ext| lower.ends_with(ext))
}

pub(crate) fn is_video_file(name: &str) -> bool {
    has_any_suffix(name, VIDEO_EXTS)
}

pub(crate) fn is_audio_file(name: &str) -> bool {
    has_any_suffix(name, AUDIO_EXTS)
}

pub(crate) fn is_image_file(name: &str) -> bool {
    has_any_suffix(name, IMAGE_EXTS)
}

pub(crate) fn is_font_file(name: &str) -> bool {
    has_any_suffix(name, FONT_EXTS)
}

pub(crate) fn is_archive_file(name: &str) -> bool {
    has_any_suffix(name, ARCHIVE_EXTS)
}

/// True when `name` looks like a video or subtitle file. Used by
/// low-confidence season rules, which must not treat arbitrary files
/// (images, fonts, notes, ...) as season evidence.
pub(crate) fn is_video_or_subtitle_file(name: &str) -> bool {
    has_any_suffix(name, SUBTITLE_EXTS) || is_video_file(name)
}
