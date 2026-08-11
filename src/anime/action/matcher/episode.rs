use lazy_regex::{Lazy, Regex, regex};

use crate::anime::action::matcher::core::MatchResult;
use crate::anime::action::matcher::ext::is_video_file;
use crate::anime::action::matcher::name::normalize_fullwidth;

/// An episode marker: a compiled regex plus how to turn capture group 1
/// into an episode number. Mirrors the season-marker table: every rule
/// reads its regex from here, so episode-marker knowledge lives in one
/// place and rules stay thin wrappers.
struct EpisodeMarker {
    re: &'static Lazy<Regex>,
    to_number: fn(&str) -> Option<u32>,
}

impl EpisodeMarker {
    /// Episode number if the marker fires on `name`. Fullwidth variants
    /// are normalized first — episode numbers are digit-sensitive and
    /// fansub names routinely use fullwidth digits (`第０１話`).
    fn number(&self, name: &str) -> Option<u32> {
        let name = normalize_fullwidth(name);
        self.re
            .captures(&name)
            .and_then(|c| c.get(1))
            .and_then(|m| (self.to_number)(m.as_str()))
    }
}

fn parse_digits(s: &str) -> Option<u32> {
    s.parse().ok()
}

static DASH_NUMBER: EpisodeMarker = EpisodeMarker {
    re: regex!(r"(?i)\s*-\s*(\d+)\s*(?:\[|\()"),
    to_number: parse_digits,
};

/// `話` (U+8A71) is the Japanese kanji used by fansub groups (e.g. 第01話);
/// `话` (U+8BDD) is Simplified Chinese, `集` is common to both.
static CHINESE_EPISODE: EpisodeMarker = EpisodeMarker {
    re: regex!(r"第(\d+)[话話集]"),
    to_number: parse_digits,
};

static BRACKETED_EPISODE: EpisodeMarker = EpisodeMarker {
    re: regex!(r"\[(\d+)\]"),
    to_number: parse_digits,
};

static TV_EPISODE: EpisodeMarker = EpisodeMarker {
    re: regex!(r"(?i)\.\s*TV\s+(\d+)"),
    to_number: parse_digits,
};

/// Marker order = the order `strip_episode_markers` tries them in.
static EPISODE_MARKERS: &[&EpisodeMarker] = &[
    &DASH_NUMBER,
    &CHINESE_EPISODE,
    &BRACKETED_EPISODE,
    &TV_EPISODE,
];

/// Rule: `- 02`-style numbers before a bracket.
pub(crate) fn match_episode_dash_number(name: &str) -> Option<MatchResult> {
    if !is_video_file(name) {
        return None;
    }
    DASH_NUMBER
        .number(name)
        .map(|n| MatchResult { number: Some(n) })
}

/// Rule: `第02話`/`第02集`-style markers.
pub(crate) fn match_episode_chinese(name: &str) -> Option<MatchResult> {
    if !is_video_file(name) {
        return None;
    }
    CHINESE_EPISODE
        .number(name)
        .map(|n| MatchResult { number: Some(n) })
}

/// Rule: bare `[02]` episode numbers. Three-digit numbers are rejected:
/// they are almost always release years (`[2024]`), not episode numbers.
pub(crate) fn match_episode_bracketed(name: &str) -> Option<MatchResult> {
    if !is_video_file(name) {
        return None;
    }
    BRACKETED_EPISODE
        .number(name)
        .filter(|&n| n < 100)
        .map(|n| MatchResult { number: Some(n) })
}

/// Rule: `.TV 02`-style markers.
pub(crate) fn match_episode_tv(name: &str) -> Option<MatchResult> {
    if !is_video_file(name) {
        return None;
    }
    TV_EPISODE
        .number(name)
        .map(|n| MatchResult { number: Some(n) })
}

/// Strip every episode marker (`- 02`, `第02話`, `[02]`, `.TV 02`) from
/// a name, e.g. `葬送的芙莉蓮 - 02` -> `葬送的芙莉蓮`. Falls back to
/// the trimmed original name if stripping leaves nothing.
#[allow(dead_code)]
pub(crate) fn strip_episode_markers(name: &str) -> String {
    let mut out = normalize_fullwidth(name.trim()).to_string();
    for marker in EPISODE_MARKERS {
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
