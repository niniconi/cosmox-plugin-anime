use crate::anime::action::matcher::{
    Confidence, ExtraKind, MatchRule, TokenKind, match_episode_bracketed, match_episode_chinese,
    match_episode_dash_number, match_episode_tv, match_extra_audio_commentary,
    match_extra_bonus_disc, match_extra_ending, match_extra_font, match_extra_generic,
    match_extra_live, match_extra_menu, match_extra_next_preview, match_extra_opening,
    match_extra_promotion, match_extra_scan, match_season_chinese, match_season_english,
    match_season_numbered_prefix, match_season_numbered_prefix_any, match_season_part,
    match_season_part_any, match_season_s_number,
};

/// Rules for files. Season matchers are gated on video/subtitle extensions
/// so arbitrary files never count as season evidence.
#[rustfmt::skip]
pub(crate) const FILE_RULES: &[MatchRule] = &[
    MatchRule { kind: TokenKind::Season, confidence: Confidence::Explicit, match_fn: match_season_s_number },
    MatchRule { kind: TokenKind::Season, confidence: Confidence::Explicit, match_fn: match_season_chinese },
    MatchRule { kind: TokenKind::Season, confidence: Confidence::Explicit, match_fn: match_season_english },
    MatchRule { kind: TokenKind::Season, confidence: Confidence::Normal,  match_fn: match_season_part },
    MatchRule { kind: TokenKind::Season, confidence: Confidence::Ambiguous, match_fn: match_season_numbered_prefix },
    // Extra rules run before episode rules so `Menu - 01`, `SP03 PV` etc.
    // are claimed as extras instead of episodes.
    MatchRule { kind: TokenKind::Extra(ExtraKind::Opening), confidence: Confidence::Explicit, match_fn: match_extra_opening },
    MatchRule { kind: TokenKind::Extra(ExtraKind::Ending), confidence: Confidence::Explicit, match_fn: match_extra_ending },
    MatchRule { kind: TokenKind::Extra(ExtraKind::NextPreview), confidence: Confidence::Explicit, match_fn: match_extra_next_preview },
    MatchRule { kind: TokenKind::Extra(ExtraKind::PromotionVideo), confidence: Confidence::Explicit, match_fn: match_extra_promotion },
    MatchRule { kind: TokenKind::Extra(ExtraKind::Live), confidence: Confidence::Explicit, match_fn: match_extra_live },
    MatchRule { kind: TokenKind::Extra(ExtraKind::Scan), confidence: Confidence::Explicit, match_fn: match_extra_scan },
    MatchRule { kind: TokenKind::Extra(ExtraKind::Font), confidence: Confidence::Explicit, match_fn: match_extra_font },
    MatchRule { kind: TokenKind::Extra(ExtraKind::Menu), confidence: Confidence::Explicit, match_fn: match_extra_menu },
    MatchRule { kind: TokenKind::Extra(ExtraKind::AudioCommentary), confidence: Confidence::Explicit, match_fn: match_extra_audio_commentary },
    MatchRule { kind: TokenKind::Extra(ExtraKind::BonusDisc), confidence: Confidence::Explicit, match_fn: match_extra_bonus_disc },
    MatchRule { kind: TokenKind::Extra(ExtraKind::Generic), confidence: Confidence::Normal, match_fn: match_extra_generic },
    MatchRule { kind: TokenKind::Episode, confidence: Confidence::Explicit, match_fn: match_episode_dash_number },
    MatchRule { kind: TokenKind::Episode, confidence: Confidence::Explicit, match_fn: match_episode_chinese },
    MatchRule { kind: TokenKind::Episode, confidence: Confidence::Normal, match_fn: match_episode_bracketed },
    MatchRule { kind: TokenKind::Episode, confidence: Confidence::Normal, match_fn: match_episode_tv },
];

/// Rules for directories. Season matchers are ungated because directory
/// names (`06. Otorimonogatari`) never pass the extension whitelist; the
/// same extra rules apply so extra directories are recognized.
#[rustfmt::skip]
pub(crate) const DIR_RULES: &[MatchRule] = &[
    MatchRule { kind: TokenKind::Season, confidence: Confidence::Explicit, match_fn: match_season_s_number },
    MatchRule { kind: TokenKind::Season, confidence: Confidence::Explicit, match_fn: match_season_chinese },
    MatchRule { kind: TokenKind::Season, confidence: Confidence::Explicit, match_fn: match_season_english },
    MatchRule { kind: TokenKind::Season, confidence: Confidence::Normal,  match_fn: match_season_part_any },
    MatchRule { kind: TokenKind::Season, confidence: Confidence::Ambiguous, match_fn: match_season_numbered_prefix_any },
    MatchRule { kind: TokenKind::Extra(ExtraKind::Opening), confidence: Confidence::Explicit, match_fn: match_extra_opening },
    MatchRule { kind: TokenKind::Extra(ExtraKind::Ending), confidence: Confidence::Explicit, match_fn: match_extra_ending },
    MatchRule { kind: TokenKind::Extra(ExtraKind::NextPreview), confidence: Confidence::Explicit, match_fn: match_extra_next_preview },
    MatchRule { kind: TokenKind::Extra(ExtraKind::PromotionVideo), confidence: Confidence::Explicit, match_fn: match_extra_promotion },
    MatchRule { kind: TokenKind::Extra(ExtraKind::Live), confidence: Confidence::Explicit, match_fn: match_extra_live },
    MatchRule { kind: TokenKind::Extra(ExtraKind::Scan), confidence: Confidence::Explicit, match_fn: match_extra_scan },
    MatchRule { kind: TokenKind::Extra(ExtraKind::Font), confidence: Confidence::Explicit, match_fn: match_extra_font },
    MatchRule { kind: TokenKind::Extra(ExtraKind::Menu), confidence: Confidence::Explicit, match_fn: match_extra_menu },
    MatchRule { kind: TokenKind::Extra(ExtraKind::AudioCommentary), confidence: Confidence::Explicit, match_fn: match_extra_audio_commentary },
    MatchRule { kind: TokenKind::Extra(ExtraKind::BonusDisc), confidence: Confidence::Explicit, match_fn: match_extra_bonus_disc },
    MatchRule { kind: TokenKind::Extra(ExtraKind::Generic), confidence: Confidence::Normal,   match_fn: match_extra_generic },
];
