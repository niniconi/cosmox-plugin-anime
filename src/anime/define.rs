use std::{collections::HashMap, fmt, ops::Deref, str::FromStr};

use serde::{Deserialize, Serialize};

use cosmox_api::api::MetadataExtend;

// Newtype wrappers that give complex field types `Display`/`FromStr`, so the
// `MetadataExtend` derive can serialize them into flat `extend` string values
// (JSON-encoded) and parse them back on read.

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TitleMap(pub HashMap<String, String>);

impl Deref for TitleMap {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for TitleMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(&self.0) {
            Ok(s) => f.write_str(&s),
            Err(_) => f.write_str("{}"),
        }
    }
}

impl FromStr for TitleMap {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map(TitleMap)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StrList(pub Vec<String>);

impl Deref for StrList {
    type Target = Vec<String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for StrList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(&self.0) {
            Ok(s) => f.write_str(&s),
            Err(_) => f.write_str("[]"),
        }
    }
}

impl FromStr for StrList {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map(StrList)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VoiceActorList(pub Vec<VoiceActor>);

impl Deref for VoiceActorList {
    type Target = Vec<VoiceActor>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for VoiceActorList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(&self.0) {
            Ok(s) => f.write_str(&s),
            Err(_) => f.write_str("[]"),
        }
    }
}

impl FromStr for VoiceActorList {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map(VoiceActorList)
    }
}

// Enums (implement Display/FromStr manually; the derive supports only structs).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AnimeStatus {
    #[default]
    Watching,
    Completed,
    OnHold,
    Dropped,
    Planned,
}

impl fmt::Display for AnimeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            AnimeStatus::Watching => "Watching",
            AnimeStatus::Completed => "Completed",
            AnimeStatus::OnHold => "OnHold",
            AnimeStatus::Dropped => "Dropped",
            AnimeStatus::Planned => "Planned",
        };
        f.write_str(s)
    }
}

impl FromStr for AnimeStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Watching" => Ok(AnimeStatus::Watching),
            "Completed" => Ok(AnimeStatus::Completed),
            "OnHold" => Ok(AnimeStatus::OnHold),
            "Dropped" => Ok(AnimeStatus::Dropped),
            "Planned" => Ok(AnimeStatus::Planned),
            _ => Err(format!("unknown AnimeStatus: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AnimeSource {
    #[default]
    Original,
    Manga,
    LightNovel,
    VisualNovel,
    Game,
    Other,
}

impl fmt::Display for AnimeSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            AnimeSource::Original => "Original",
            AnimeSource::Manga => "Manga",
            AnimeSource::LightNovel => "LightNovel",
            AnimeSource::VisualNovel => "VisualNovel",
            AnimeSource::Game => "Game",
            AnimeSource::Other => "Other",
        };
        f.write_str(s)
    }
}

impl FromStr for AnimeSource {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Original" => Ok(AnimeSource::Original),
            "Manga" => Ok(AnimeSource::Manga),
            "LightNovel" => Ok(AnimeSource::LightNovel),
            "VisualNovel" => Ok(AnimeSource::VisualNovel),
            "Game" => Ok(AnimeSource::Game),
            "Other" => Ok(AnimeSource::Other),
            _ => Err(format!("unknown AnimeSource: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Other,
}

impl fmt::Display for Gender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Gender::Male => "Male",
            Gender::Female => "Female",
            Gender::Other => "Other",
        };
        f.write_str(s)
    }
}

impl FromStr for Gender {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Male" => Ok(Gender::Male),
            "Female" => Ok(Gender::Female),
            "Other" => Ok(Gender::Other),
            _ => Err(format!("unknown Gender: {s}")),
        }
    }
}

// Anime shared structures (key prefix `anime`).

#[derive(Debug, Default, Serialize, Deserialize, MetadataExtend)]
#[extend(key = "anime")]
pub struct Anime {
    /// Full multi-language title mapping (`main` / `en` / `ja` / ...).
    /// The primary title lives in the node's `name` field.
    pub titles: TitleMap,
    pub genres: StrList,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: AnimeStatus,
    pub source: AnimeSource,
    pub studios: StrList,
    pub rating: Option<f32>,
    /// Official website of the series.
    pub official_website: Option<String>,
    /// Number of episodes, when the source reports it.
    pub eps: Option<u32>,
    /// Broadcasting platform (TV / Web / Movie ...).
    pub platform: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, MetadataExtend)]
#[extend(key = "anime-season")]
pub struct Season {
    pub season_number: Option<u32>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, MetadataExtend)]
#[extend(key = "anime-episode")]
pub struct Episode {
    pub season_number: Option<u32>,
    pub episode_number: Option<u32>,
    pub air_date: Option<String>,
    pub duration_minutes: Option<f32>,
    pub extra_kind: Option<String>,
    pub extra_title: Option<String>,
    pub extra_tag: Option<String>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EpisodeContainer {
    pub season_number: Option<u32>,
    pub episode: Episode,
}

#[derive(Debug, Default, Serialize, Deserialize, MetadataExtend)]
#[extend(key = "anime-character")]
pub struct Character {
    pub voice_actors: VoiceActorList,
    pub gender: Option<Gender>,
    pub birth_date: Option<String>,
    /// Role of the character in the work (main / supporting / ...).
    pub role: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VoiceActor {
    pub id: u32,
    pub name: String,
    pub name_japanese: Option<String>,
    pub birth_date: Option<String>,
    pub nationality: Option<String>,
    pub image_url: Option<String>,
}

// Staff shared structure (key prefix `anime-staff`).

#[derive(Debug, Default, Serialize, Deserialize, MetadataExtend)]
#[extend(key = "anime-staff")]
pub struct Staff {
    pub career: StrList,
    pub episode_range: Option<String>,
}
