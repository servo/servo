use std::time;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Metadata {
    pub duration: Option<time::Duration>,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub is_seekable: bool,
    // TODO: Might be nice to move width and height along with each video track.
    pub video_tracks: Vec<String>,
    pub audio_tracks: Vec<String>,
    // Whether the media comes from a live source or not.
    pub is_live: bool,
    pub title: Option<String>,
}
