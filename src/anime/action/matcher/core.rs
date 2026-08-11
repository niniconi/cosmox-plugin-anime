use std::cmp::Ordering;

/// Confidence level for a match.
///
/// Ordering is `Explicit > Normal > Ambiguous` (see the [`Ord`] impl).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    Ambiguous,
    Normal,
    Explicit,
}

impl PartialOrd for Confidence {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Confidence {
    fn cmp(&self, other: &Self) -> Ordering {
        fn rank(c: Confidence) -> u8 {
            match c {
                Confidence::Ambiguous => 0,
                Confidence::Normal => 1,
                Confidence::Explicit => 2,
            }
        }
        rank(*self).cmp(&rank(*other))
    }
}

/// What semantic kind a match function targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Season,
    Episode,
    Extra(ExtraKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtraKind {
    Opening,
    Ending,
    InsertSong,
    Credits,
    Menu,
    VideoExtra,
    PromotionVideo,
    NextPreview,
    Live,
    AudioCommentary,
    BonusDisc,
    Scan,
    Booklet,
    Artwork,
    Poster,
    Subtitle,
    Font,
    Chapter,
    Thumbnail,
    DataFile,
    Patch,
    Generic,
    Unknown,
}

pub type MatchFn = fn(&str) -> Option<MatchResult>;

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub number: Option<u32>,
}

/// A value tagged with its matching confidence.
///
/// Used where several matches compete for the same field so the strongest
/// one can be selected (see [`Scored::pick`]).
#[derive(Debug, Clone)]
pub struct Scored<T> {
    pub value: T,
    pub confidence: Confidence,
}

impl<T> Scored<T> {
    /// Pick the value with the higher confidence.
    ///
    /// Returns the higher-confidence argument; if both are present and
    /// equally confident, `a` wins. Used when merging information from
    /// parent and child nodes.
    pub fn pick(a: Option<Self>, b: Option<Self>) -> Option<Self> {
        match (a, b) {
            (Some(a), Some(b)) => {
                if a.confidence >= b.confidence {
                    Some(a)
                } else {
                    Some(b)
                }
            }
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }
}

/// A single matching rule.
///
/// Rules are evaluated in priority order. The first match for each
/// [`TokenKind`] wins; later rules of the same kind are skipped.
pub struct MatchRule {
    /// What kind of content this rule claims to identify.
    pub kind: TokenKind,
    /// How reliable a match this rule produces.
    pub confidence: Confidence,
    /// The matcher that decides whether the rule fires on a name.
    pub match_fn: MatchFn,
}

/// Complete parsed information for a single tree node (file or directory).
///
/// `season`, `episode` and `kind` are populated by the first matching rule of
/// the corresponding [`TokenKind`]. `title` is reserved for a future
/// cleaned-up display name; no rule populates it today and it is only
/// propagated through [`ParsedInfo::inherit`].
#[derive(Debug, Default, Clone)]
pub struct ParsedInfo {
    pub season: Option<Scored<u32>>,
    pub episode: Option<Scored<u32>>,
    pub kind: Option<Scored<TokenKind>>,
    pub title: String,
}

impl ParsedInfo {
    /// Inherit fields from a parent node.
    ///
    /// The higher-confidence value wins; on equal confidence, the child
    /// wins. The episode is never inherited.
    #[must_use]
    pub fn inherit(parent: &ParsedInfo, child: &ParsedInfo) -> ParsedInfo {
        ParsedInfo {
            season: Scored::pick(child.season.clone(), parent.season.clone()),
            episode: child.episode.clone(),
            kind: Scored::pick(child.kind.clone(), parent.kind.clone()),
            title: if child.title.is_empty() {
                parent.title.clone()
            } else {
                child.title.clone()
            },
        }
    }
}

/// Run the full rule list against `name` and return the combined result.
///
/// Each [`TokenKind`] accepts at most one match — the first rule of that
/// kind that fires wins.
#[must_use]
pub fn parse_info(name: &str, rules: &[MatchRule]) -> ParsedInfo {
    let mut info = ParsedInfo::default();

    for rule in rules {
        match rule.kind {
            TokenKind::Season if info.season.is_some() => continue,
            TokenKind::Episode if info.episode.is_some() => continue,
            TokenKind::Extra(_) if info.kind.is_some() => continue,
            _ => {}
        }

        let Some(result) = (rule.match_fn)(name) else {
            continue;
        };

        match rule.kind {
            TokenKind::Season => {
                if let Some(n) = result.number {
                    info.season = Some(Scored {
                        value: n,
                        confidence: rule.confidence,
                    });
                }
            }
            TokenKind::Episode => {
                if let Some(n) = result.number {
                    info.episode = Some(Scored {
                        value: n,
                        confidence: rule.confidence,
                    });
                }
            }
            TokenKind::Extra(k) => {
                info.kind = Some(Scored {
                    value: TokenKind::Extra(k),
                    confidence: rule.confidence,
                });
            }
        }
    }

    info
}
