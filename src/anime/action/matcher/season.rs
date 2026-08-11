use lazy_regex::{Lazy, Regex, regex};

use crate::anime::action::matcher::core::MatchResult;
use crate::anime::action::matcher::ext::is_video_or_subtitle_file;
use crate::anime::action::matcher::name::normalize_fullwidth;

/// A season marker: a compiled regex plus how to turn capture group 1
/// into a season number (plain digits, Chinese numerals, or Roman).
///
/// This table is the single source of truth for season-marker knowledge:
/// the `match_season_*` rules (classification) and `strip_season_markers`
/// (name normalization for merging) both read from it, so adding a new
/// marker touches exactly one place.
struct SeasonMarker {
    re: &'static Lazy<Regex>,
    to_number: fn(&str) -> Option<u32>,
}

impl SeasonMarker {
    /// Season number if the marker fires on `name`.
    fn number(&self, name: &str) -> Option<u32> {
        self.re
            .captures(name)
            .and_then(|c| c.get(1))
            .and_then(|m| (self.to_number)(m.as_str()))
    }
}

fn parse_digits(s: &str) -> Option<u32> {
    s.parse().ok()
}

fn parse_chinese_digits(s: &str) -> Option<u32> {
    s.parse::<u32>().ok().or_else(|| chinese_digit(s))
}

fn parse_roman_digits(s: &str) -> Option<u32> {
    s.parse::<u32>().ok().or_else(|| roman_digit(s))
}

static S_NUMBER: SeasonMarker = SeasonMarker {
    re: regex!(r"(?i)(?:^|\W)s(\d+)(?:\W|$)"),
    to_number: parse_digits,
};
static CHINESE_SEASON: SeasonMarker = SeasonMarker {
    re: regex!(r"第(\d+|[一二三四五六七八九十百零])[季期]"),
    to_number: parse_chinese_digits,
};
static ENGLISH_SEASON: SeasonMarker = SeasonMarker {
    re: regex!(r"(?i)(?:^|\W)season\s*(\d+)(?:\W|$)"),
    to_number: parse_digits,
};
static PART_MARKER: SeasonMarker = SeasonMarker {
    re: regex!(r"(?i)(?:^|\W)part\s*(\d+|i[vx]|vi|iv|v?i{1,3})(?:\W|$)"),
    to_number: parse_roman_digits,
};
static NUMBERED_PREFIX: SeasonMarker = SeasonMarker {
    re: regex!(r"^(\d+)\.(?:\s|\D)"),
    to_number: parse_digits,
};

/// Marker order = the order `strip_season_markers` tries them in.
static SEASON_MARKERS: &[&SeasonMarker] = &[
    &S_NUMBER,
    &CHINESE_SEASON,
    &ENGLISH_SEASON,
    &PART_MARKER,
    &NUMBERED_PREFIX,
];

/// Rule: `S2`-style season markers.
pub(crate) fn match_season_s_number(name: &str) -> Option<MatchResult> {
    S_NUMBER
        .number(name)
        .map(|n| MatchResult { number: Some(n) })
}

/// Rule: `第2季`/`第1期`-style season markers.
pub(crate) fn match_season_chinese(name: &str) -> Option<MatchResult> {
    CHINESE_SEASON
        .number(name)
        .map(|n| MatchResult { number: Some(n) })
}

/// Rule: `Season 2`-style season markers.
pub(crate) fn match_season_english(name: &str) -> Option<MatchResult> {
    ENGLISH_SEASON
        .number(name)
        .map(|n| MatchResult { number: Some(n) })
}

/// Rule: `Part II`-style season markers on video or subtitle files.
pub(crate) fn match_season_part(name: &str) -> Option<MatchResult> {
    if !is_video_or_subtitle_file(name) {
        return None;
    }
    PART_MARKER
        .number(name)
        .map(|n| MatchResult { number: Some(n) })
}

/// Rule: leading `2. ` numeric prefixes on video or subtitle files.
pub(crate) fn match_season_numbered_prefix(name: &str) -> Option<MatchResult> {
    if !is_video_or_subtitle_file(name) {
        return None;
    }
    NUMBERED_PREFIX
        .number(name)
        .map(|n| MatchResult { number: Some(n) })
}

/// Ungated variant of `match_season_part` for directory names, which never
/// pass the video/subtitle extension whitelist (`06. Otorimonogatari`).
pub(crate) fn match_season_part_any(name: &str) -> Option<MatchResult> {
    PART_MARKER
        .number(name)
        .map(|n| MatchResult { number: Some(n) })
}

/// Ungated variant of `match_season_numbered_prefix` for directory names.
pub(crate) fn match_season_numbered_prefix_any(name: &str) -> Option<MatchResult> {
    NUMBERED_PREFIX
        .number(name)
        .map(|n| MatchResult { number: Some(n) })
}

fn chinese_digit(s: &str) -> Option<u32> {
    match s {
        "零" => Some(0),
        "一" => Some(1),
        "二" => Some(2),
        "三" => Some(3),
        "四" => Some(4),
        "五" => Some(5),
        "六" => Some(6),
        "七" => Some(7),
        "八" => Some(8),
        "九" => Some(9),
        "十" => Some(10),
        "百" => Some(100),
        _ => None,
    }
}

fn roman_digit(s: &str) -> Option<u32> {
    match s.to_lowercase().as_str() {
        "i" => Some(1),
        "ii" => Some(2),
        "iii" => Some(3),
        "iv" => Some(4),
        "v" => Some(5),
        "vi" => Some(6),
        "vii" => Some(7),
        "viii" => Some(8),
        "ix" => Some(9),
        "x" => Some(10),
        _ => None,
    }
}

/// Strip every season marker (`S2`, `Season 2`, `第二季`, `Part II`,
/// leading `2. `) from a series name so same-title series can be merged,
/// e.g. `葬送的芙莉蓮` + `葬送的芙莉蓮 S2` -> both become `葬送的芙莉蓮`.
/// Falls back to the trimmed original name if stripping leaves nothing.
pub(crate) fn strip_season_markers(name: &str) -> String {
    let mut out = normalize_fullwidth(name.trim()).to_string();
    for marker in SEASON_MARKERS {
        if marker.re.is_match(&out) {
            out = marker.re.replace(&out, "").into_owned();
        }
    }
    let cleaned = out.split_whitespace().collect::<Vec<_>>().join(" ");
    if cleaned.is_empty() {
        name.trim().to_string()
    } else {
        cleaned
    }
}
