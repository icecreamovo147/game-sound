use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackMode {
    #[default]
    Overlay,
    Interrupt,
    Queue,
    Exclusive,
}

impl PlaybackMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Overlay => "overlay",
            Self::Interrupt => "interrupt",
            Self::Queue => "queue",
            Self::Exclusive => "exclusive",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sound {
    pub id: i64,
    pub name: String,
    pub file_path: String,
    pub category_id: Option<i64>,
    pub profile_id: Option<i64>,
    pub volume: f32,
    pub playback_mode: PlaybackMode,
    pub loop_enabled: bool,
    pub favorite: bool,
    pub tags: String,
    pub note: String,
    pub sort_order: i64,
    pub play_count: i64,
    pub last_played_at: Option<String>,
}
impl Sound {
    pub fn is_available(&self) -> bool {
        Path::new(&self.file_path).is_file()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub profile_id: Option<i64>,
    pub sort_order: i64,
}
