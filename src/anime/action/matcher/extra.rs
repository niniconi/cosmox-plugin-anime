use lazy_regex::regex_captures;

use crate::anime::action::matcher::core::{ExtraKind, MatchResult};
use crate::anime::action::matcher::ext::{
    is_archive_file, is_audio_file, is_font_file, is_image_file, is_video_or_subtitle_file,
};

/// Gate helpers.
///
/// Every extra match function must answer "does this name plausibly carry
/// this kind of content?" A directory name (no extension, or a numeric
/// suffix like `Vol.09`) always passes — the kind is decided by the
/// directory itself and inherited by its children. A file must match the
/// media type of the bucket, so a video never becomes a scan and a plain
/// `.md5` never becomes an extra.
fn is_dir_name(name: &str) -> bool {
    let Some((_, ext)) = name.rsplit_once('.') else {
        return true;
    };
    ext.chars().all(|c| c.is_ascii_digit())
}

fn video_ok(name: &str) -> bool {
    is_video_or_subtitle_file(name) || is_dir_name(name)
}

fn image_or_archive_ok(name: &str) -> bool {
    is_image_file(name) || is_archive_file(name) || is_dir_name(name)
}

fn disc_data_ok(name: &str) -> bool {
    if is_audio_file(name) || is_archive_file(name) || is_dir_name(name) {
        return true;
    }
    // Sidecar files that always travel with a CD image (cue/bin/img/log).
    let lower = name.to_lowercase();
    [".cue", ".bin", ".img", ".log"]
        .iter()
        .any(|ext| lower.ends_with(ext))
}

fn generic_video_ok(name: &str) -> bool {
    is_video_or_subtitle_file(name)
        || is_audio_file(name)
        || is_archive_file(name)
        || is_dir_name(name)
}

/// Extract the leading number of an extra item, e.g. `PV - 02` -> 2,
/// `SP03` -> 3, `Menu - 01` -> 1. Used for `MatchResult::number` so the
/// frontend can order extras within a kind.
fn extra_number(name: &str, pattern: &str) -> Option<u32> {
    lazy_regex::Regex::new(pattern)
        .ok()?
        .captures(name)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

/// OP / NCOP (non-credit opening) videos and subtitles.
pub(crate) fn match_extra_opening(name: &str) -> Option<MatchResult> {
    if !video_ok(name) {
        return None;
    }
    if regex_captures!(
        r"(?i)(?:\bNCOP\d*\b|\bOP\d*\b|Opening|オープニング|片头曲|片頭曲|片头|片頭|主题曲|主題曲)",
        name
    )
    .is_some()
    {
        let number = extra_number(name, r"(?i)(?:OP|NCOP)[\s._-]*(\d+)");
        return Some(MatchResult { number });
    }
    None
}

/// ED / NCED (non-credit ending) videos and subtitles.
pub(crate) fn match_extra_ending(name: &str) -> Option<MatchResult> {
    if !video_ok(name) {
        return None;
    }
    if regex_captures!(
        r"(?i)(?:\bNCED\d*\b|\bED\d*\b|Ending|エンディング|片尾曲|片尾)",
        name
    )
    .is_some()
    {
        let number = extra_number(name, r"(?i)(?:ED|NCED)[\s._-]*(\d+)");
        return Some(MatchResult { number });
    }
    None
}

/// PV (promotional video), CM (commercial), teaser and previews.
pub(crate) fn match_extra_promotion(name: &str) -> Option<MatchResult> {
    if !video_ok(name) {
        return None;
    }
    if regex_captures!(
        r"(?i)(?:\bPV\d*\b|\bCM\d*\b|Teaser|Promo|Preview|特报|特報|予告|宣传|宣傳|预告|預告)",
        name
    )
    .is_some()
    {
        let number = extra_number(name, r"(?i)(?:PV|CM|Teaser)[\s._-]*(\d+)");
        return Some(MatchResult { number });
    }
    None
}

/// Music videos, live concerts and character song videos.
pub(crate) fn match_extra_live(name: &str) -> Option<MatchResult> {
    if !is_video_or_subtitle_file(name) && !is_audio_file(name) && !is_dir_name(name) {
        return None;
    }
    if regex_captures!(
        r"(?i)(?:\bMV\b|Music Video|MusicVideo|Live|ライブ|Concert|演唱会|演唱會|コンサート)",
        name
    )
    .is_some()
    {
        let number = extra_number(name, r"(?i)(?:MV|Live)[\s._-]*(\d+)");
        return Some(MatchResult { number });
    }
    None
}

/// BD/DVD scans: covers, booklets, artwork, posters — images or archives.
pub(crate) fn match_extra_scan(name: &str) -> Option<MatchResult> {
    if !image_or_archive_ok(name) {
        return None;
    }
    if regex_captures!(
        r"(?i)(?:\bScans?\b|Booklet|ブックレット|Artbook|Art Book|設定集|设定集|画集|畫集|原案|Storyboard|分镜|分鏡|ジャケット|ポスター|Poster|Cover|ケース|掃图|掃圖|扫图|スキャン|BDBOX|Box Set|Official Guidebook|Production Note|Illustrated Book|BD Scan|BD Cover)",
        name
    )
    .is_some()
    {
        let number = extra_number(name, r"(?i)(?:Scan|Vol)[\s._-]*(\d+)");
        return Some(MatchResult { number });
    }
    None
}

/// Font files shipped with the release.
pub(crate) fn match_extra_font(name: &str) -> Option<MatchResult> {
    if !is_font_file(name) && !is_archive_file(name) && !is_dir_name(name) {
        return None;
    }
    if regex_captures!(r"(?i)(?:\bFont\b|\bFonts\b|字体|フォント)", name).is_some()
        || is_font_file(name)
    {
        return Some(MatchResult { number: None });
    }
    None
}

/// BD/DVD menu videos.
pub(crate) fn match_extra_menu(name: &str) -> Option<MatchResult> {
    if !is_video_or_subtitle_file(name) && !is_dir_name(name) {
        return None;
    }
    if regex_captures!(
        r"(?i)(?:\bMenu\b|\bMenus\b|メニュー|菜单|選單|BD Menu)",
        name
    )
    .is_some()
    {
        let number = extra_number(name, r"(?i)Menu[\s._-]*(\d+)");
        return Some(MatchResult { number });
    }
    None
}

/// Audio commentary tracks (video or audio).
pub(crate) fn match_extra_audio_commentary(name: &str) -> Option<MatchResult> {
    if !is_video_or_subtitle_file(name) && !is_audio_file(name) && !is_dir_name(name) {
        return None;
    }
    if regex_captures!(
        r"(?i)(?:Audio Commentary|Audio commentary|Commentary|音声评论|音聲評論|コメンタリー)",
        name
    )
    .is_some()
    {
        return Some(MatchResult { number: None });
    }
    None
}

/// Bonus CDs: OSTs, drama CDs, character songs, radio — audio or archives.
pub(crate) fn match_extra_bonus_disc(name: &str) -> Option<MatchResult> {
    if !disc_data_ok(name) {
        return None;
    }
    if regex_captures!(
        r"(?i)(?:\bCDs?\b|Bonus CD|特典CD|CD Vol|CDImage|Drama CD|ドラマCD|广播剧|廣播劇|OST|Soundtrack|サントラ|Character Song|角色歌|キャラソン|Radio|ラジオ|Interview|访谈|訪談|Special CD|SPCD|Music Collection|Original Soundtrack)",
        name
    )
    .is_some()
    {
        let number = extra_number(name, r"(?i)(?:CD|Vol)[\s._-]*(\d+)");
        return Some(MatchResult { number });
    }
    None
}

/// Next-episode previews (次回予告).
pub(crate) fn match_extra_next_preview(name: &str) -> Option<MatchResult> {
    if !is_video_or_subtitle_file(name) && !is_dir_name(name) {
        return None;
    }
    if regex_captures!(
        r"(?i)(?:次回予告|NextPreview|Next Preview|Next Episode Preview|次回)",
        name
    )
    .is_some()
    {
        let number = extra_number(name, r"(?i)(?:Preview)[\s._-]*(\d+)");
        return Some(MatchResult { number });
    }
    None
}

/// Generic specials: `SPxx`, SPs, digest, summary, OVA extras — anything
/// clearly extra content that does not fit a more specific category.
pub(crate) fn match_extra_generic(name: &str) -> Option<MatchResult> {
    if !generic_video_ok(name) {
        return None;
    }
    if regex_captures!(
        r"(?i)(?:\bSP\d*\b|\bSPs\b|Special|特典|映像特典|Digest|Summary|Tokuten|スペシャル|Extra|特番|OVA特典)",
        name
    )
    .is_some()
    {
        let number = extra_number(name, r"(?i)SP[\s._-]*(\d+)");
        return Some(MatchResult { number });
    }
    None
}

/// Map an `ExtraKind` to the display name of its bucket directory.
pub(crate) fn extra_bucket_name(kind: ExtraKind) -> &'static str {
    match kind {
        ExtraKind::Opening => "Opening",
        ExtraKind::Ending => "Ending",
        ExtraKind::PromotionVideo => "PV & CM",
        ExtraKind::Live => "Live & MV",
        ExtraKind::Scan => "Scans",
        ExtraKind::Font => "Fonts",
        ExtraKind::Menu => "Menus",
        ExtraKind::AudioCommentary => "Audio Commentary",
        ExtraKind::BonusDisc => "Bonus Discs",
        ExtraKind::NextPreview => "Next Previews",
        _ => "Extras",
    }
}

/// Display title for an extra file: extension stripped, brackets removed,
/// underscores replaced with spaces. `series_title_from_name` is not used
/// because it strips release tags that carry meaning for extras
/// (`[SP01]`, `(flac)`).
pub(crate) fn extra_title_from_name(name: &str) -> String {
    let stem = name.rsplit_once('.').map(|(s, _)| s).unwrap_or(name);
    stem.replace(['[', ']'], "")
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Content tag for scan-like images: booklet / cover / artwork /
/// storyboard / poster. Only applied to actual image files.
pub(crate) fn extra_tag_from_name(name: &str, kind: ExtraKind) -> Option<String> {
    if !matches!(
        kind,
        ExtraKind::Scan | ExtraKind::Booklet | ExtraKind::Artwork | ExtraKind::Poster
    ) || !is_image_file(name)
    {
        return None;
    }
    let lower = name.to_lowercase();
    let tag = if lower.contains("booklet")
        || lower.contains("ブックレット")
        || lower.contains("小説")
    {
        "booklet"
    } else if lower.contains("storyboard") || lower.contains("分镜") || lower.contains("分鏡") {
        "storyboard"
    } else if lower.contains("poster") || lower.contains("ポスター") {
        "poster"
    } else if lower.contains("artwork")
        || lower.contains("原画")
        || lower.contains("原畫")
        || lower.contains("イラスト")
        || lower.contains("art book")
    {
        "artwork"
    } else if lower.contains("cover")
        || lower.contains("case")
        || lower.contains("jacket")
        || lower.contains("ジャケット")
        || lower.contains("表")
        || lower.contains("digipack")
        || lower.contains("bd")
    {
        "cover"
    } else {
        return None;
    };
    Some(tag.to_string())
}

/// Stable snake_case identifier for a kind, stored as the `:extra_kind`
/// annotation so the frontend can filter without parsing display names.
pub(crate) fn extra_kind_str(kind: ExtraKind) -> &'static str {
    match kind {
        ExtraKind::Opening => "opening",
        ExtraKind::Ending => "ending",
        ExtraKind::InsertSong => "insert_song",
        ExtraKind::Credits => "credits",
        ExtraKind::Menu => "menu",
        ExtraKind::VideoExtra => "video_extra",
        ExtraKind::PromotionVideo => "promotion_video",
        ExtraKind::NextPreview => "next_preview",
        ExtraKind::Live => "live",
        ExtraKind::AudioCommentary => "audio_commentary",
        ExtraKind::BonusDisc => "bonus_disc",
        ExtraKind::Scan => "scan",
        ExtraKind::Booklet => "booklet",
        ExtraKind::Artwork => "artwork",
        ExtraKind::Poster => "poster",
        ExtraKind::Subtitle => "subtitle",
        ExtraKind::Font => "font",
        ExtraKind::Chapter => "chapter",
        ExtraKind::Thumbnail => "thumbnail",
        ExtraKind::DataFile => "data_file",
        ExtraKind::Patch => "patch",
        ExtraKind::Generic => "generic",
        ExtraKind::Unknown => "unknown",
    }
}
