use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// the stream
/// A stream of audio maintained by the Snapcast server
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Stream {
  pub id: String,
  pub properties: Option<StreamProperties>,
  pub status: StreamStatus,
  pub uri: StreamUri,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum StreamStatus {
  Idle,
  Playing,
  Disabled,
  Unknown,
}

impl From<&str> for StreamStatus {
  fn from(s: &str) -> Self {
    match s {
      "idle" => StreamStatus::Idle,
      "playing" => StreamStatus::Playing,
      "disabled" => StreamStatus::Disabled,
      _ => StreamStatus::Unknown,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreamUri {
  pub fragment: String,
  pub host: String,
  pub path: String,
  pub query: HashMap<String, String>,
  pub raw: String,
  pub scheme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum StreamPlaybackStatus {
  Playing,
  Paused,
  Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum StreamLoopStatus {
  None,
  Track,
  Playlist,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StreamProperties {
  pub playback_status: Option<StreamPlaybackStatus>,
  pub loop_status: Option<StreamLoopStatus>,
  pub shuffle: Option<bool>,
  pub volume: Option<usize>,
  pub mute: Option<bool>,
  pub rate: Option<f64>,
  pub position: Option<f64>,
  pub can_go_next: bool,
  pub can_go_previous: bool,
  pub can_play: bool,
  pub can_pause: bool,
  pub can_seek: bool,
  pub can_control: bool,
  pub metadata: Option<StreamMetadata>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StreamMetadata {
  track_id: Option<String>,
  file: Option<String>,
  duration: Option<f64>,
  artist: Option<Vec<String>>,
  artist_sort: Option<Vec<String>>,
  album: Option<String>,
  album_sort: Option<String>,
  album_artist: Option<Vec<String>>,
  album_artist_sort: Option<Vec<String>>,
  name: Option<String>,
  date: Option<String>,
  original_date: Option<String>,
  composer: Option<Vec<String>>,
  performer: Option<String>,
  work: Option<String>,
  grouping: Option<String>,
  label: Option<String>,
  musicbrainz_artist_id: Option<String>,
  musicbrainz_album_id: Option<String>,
  musicbrainz_album_artist_id: Option<String>,
  musicbrainz_track_id: Option<String>,
  musicbrainz_release_track_id: Option<String>,
  musicbrainz_work_id: Option<String>,
  lyrics: Option<Vec<String>>,
  bpm: Option<usize>,
  auto_rating: Option<f64>,
  comment: Option<Vec<String>>,
  content_created: Option<String>,
  disc_number: Option<usize>,
  first_used: Option<String>,
  genre: Option<Vec<String>>,
  last_used: Option<String>,
  lyricist: Option<Vec<String>>,
  title: Option<String>,
  track_number: Option<usize>,
  url: Option<String>,
  art_url: Option<String>,
  art_data: Option<ArtData>,
  use_count: Option<usize>,
  user_rating: Option<f64>,
  spotify_artist_id: Option<String>,
  spotify_track_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtData {
  pub data: String,
  pub extension: String,
}

// params and results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddStreamParams {
  #[serde(rename = "streamUri")]
  pub stream_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddStreamResult {
  pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RemoveStreamParams {
  pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RemoveStreamResult {
  pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ControlParams {
  pub id: String,
  #[serde(flatten)]
  pub command: ControlCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "command", content = "params")]
pub enum ControlCommand {
  Play,
  Pause,
  PlayPause,
  Stop,
  Next,
  Previous,
  Seek { offset: f64 },
  SetPosition { position: f64 },
}

pub type ControlResult = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetPropertyParams {
  pub id: String,
  #[serde(flatten)]
  pub properties: SetPropertyProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "property", content = "value")]
pub enum SetPropertyProperties {
  LoopStatus(StreamLoopStatus),
  Shuffle(bool),
  Volume(usize),
  Mute(bool),
  Rate(f64),
}

pub type SetPropertiesResult = String;

// notifications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnUpdateParams {
  pub id: String,
  pub stream: Stream,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnPropertiesParams {
  pub id: String,
  pub properties: StreamProperties,
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use super::*;

  #[test]
  fn serialize_stream() {
    let stream = Stream {
      id: "stream 1".to_string(),
      status: "idle".into(),
      properties: None,
      uri: StreamUri {
        fragment: "".to_string(),
        host: "".to_string(),
        path: "/tmp/snapfifo".to_string(),
        query: HashMap::new(),
        raw: "pipe:///tmp/snapfifo?name=stream 1".to_string(),
        scheme: "pipe".to_string(),
      },
    };

    let json = serde_json::to_string(&stream).unwrap();
    let expected = r#"{"id":"stream 1","status":"idle","uri":{"fragment":"","host":"","path":"/tmp/snapfifo","query":{},"raw":"pipe:///tmp/snapfifo?name=stream 1","scheme":"pipe"}}"#;

    assert_eq!(json, expected);
  }

  #[test]
  fn deserialize_stream() {
    let json = r#"{"id":"stream 1","status":"idle","uri":{"fragment":"","host":"","path":"/tmp/snapfifo","query":{"chunk_ms":"20","codec":"flac","name":"stream 1","sampleformat":"48000:16:2"},"raw":"pipe:///tmp/snapfifo?name=stream 1","scheme":"pipe"}}"#;
    let stream: Stream = serde_json::from_str(json).unwrap();

    assert_eq!(stream.id, "stream 1");
  }
}
