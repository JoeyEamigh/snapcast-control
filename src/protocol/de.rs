use dashmap::DashMap;
use serde::de::{DeserializeSeed, MapAccess, Visitor};
use std::collections::HashMap;
use uuid::Uuid;

use super::{notification::NotificationMethodConverter, request::RequestMethod, result::SnapcastResult};
use crate::Message;

pub type SentRequests = DashMap<Uuid, RequestMethod>;
pub struct SnapcastDeserializer<'a>(&'a SentRequests);

impl<'a> SnapcastDeserializer<'a> {
  pub fn de(message: &str, state: &'a SentRequests) -> Result<Message, DeserializationError> {
    let mut deserializer = serde_json::Deserializer::from_str(message);

    Ok(SnapcastDeserializer(state).deserialize(&mut deserializer)?)
  }
}

impl<'a> TryFrom<(&'a str, &'a SentRequests)> for Message {
  type Error = DeserializationError;

  fn try_from(
    (message, state): (&'a str, &'a SentRequests),
  ) -> Result<Self, <crate::protocol::Message as TryFrom<(&'a str, &'a SentRequests)>>::Error> {
    SnapcastDeserializer::de(message, state)
  }
}

impl<'de, 'a> DeserializeSeed<'de> for SnapcastDeserializer<'a> {
  type Value = Message;

  fn deserialize<D>(self, d: D) -> Result<Self::Value, D::Error>
  where
    D: serde::de::Deserializer<'de>,
  {
    struct SnapcastDeserializerVisitor<'a>(&'a SentRequests);

    impl<'de> Visitor<'de> for SnapcastDeserializerVisitor<'_> {
      type Value = Message;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a valid snapcast jsonrpc message")
      }

      fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
      where
        A: MapAccess<'de>,
      {
        use serde::de::Error;
        use serde_json::Value;

        let mut response: HashMap<String, Value> = HashMap::new();

        while let Some((key, value)) = access.next_entry()? {
          tracing::trace!("map key {:?} => {:?}", key, value);
          response.insert(key, value);
        }

        let jsonrpc = response
          .get("jsonrpc")
          .unwrap_or(&Value::String("2.0".to_string()))
          .as_str()
          .unwrap_or("2.0")
          .to_string();

        if response.contains_key("method") {
          Ok(Message::Notification {
            jsonrpc,
            method: Box::new(
              NotificationMethodConverter(
                serde_json::from_value(response.remove("method").expect("this should never fail"))
                  .map_err(Error::custom)?,
                response.remove("params").ok_or(Error::custom("no response found??"))?,
              )
              .try_into()
              .map_err(Error::custom)?,
            ),
          })
        } else if response.contains_key("result") {
          let id: Uuid = serde_json::from_value(
            response
              .remove("id")
              .ok_or(Error::custom("could not associate result with request"))?,
          )
          .map_err(Error::custom)?;
          let result = response.remove("result").expect("this should never fail");
          let result = if let Some(mapped_type) = self.0.remove(&id) {
            SnapcastResult::try_from((mapped_type.1, result)).map_err(Error::custom)?
          } else {
            serde_json::from_value(result).map_err(Error::custom)?
          };

          Ok(Message::Result {
            id,
            jsonrpc,
            result: Box::new(result),
          })
        } else if response.contains_key("error") {
          let id: Uuid = serde_json::from_value(
            response
              .remove("id")
              .ok_or(Error::custom("could not associate result with request"))?,
          )
          .map_err(Error::custom)?;
          Ok(Message::Error {
            id,
            jsonrpc,
            error: serde_json::from_value(response.remove("error").expect("this should never fail"))
              .map_err(Error::custom)?,
          })
        } else {
          Err(Error::custom("invalid snapcast message"))
        }
      }
    }

    d.deserialize_map(SnapcastDeserializerVisitor(self.0))
  }
}

/// Errors that can occur during deserialization
#[derive(Debug, thiserror::Error)]
pub enum DeserializationError {
  /// general deserialization error
  #[error("Deserialization error: {0}")]
  DeserializationError(#[from] serde::de::value::Error),
  /// json deserialization error
  #[error("JSON Deserialization error: {0}")]
  SerdeJsonError(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
  use crate::protocol::{client, group, Method, Notification, Request, SnapcastResult};

  use super::*;

  #[test]
  fn deserialize_error() {
    let map = DashMap::new();

    let message = r#"{"id": "00000000-0000-0000-0000-000000000000", "jsonrpc": "2.0", "error": {"code": -32603, "message": "Internal error"}}"#;
    let snapcast_message = SnapcastDeserializer::de(message, &map).unwrap();

    assert_eq!(
      snapcast_message,
      Message::Error {
        id: "00000000-0000-0000-0000-000000000000".try_into().unwrap(),
        jsonrpc: "2.0".to_string(),
        error: serde_json::from_str(r#"{"code": -32603, "message": "Internal error"}"#).unwrap()
      }
    );
  }

  #[test]
  fn serialize_client_get_status() {
    let message = r#"{"id":"00000000-0000-0000-0000-000000000000","jsonrpc":"2.0","method":"Client.GetStatus","params":{"id":"00:21:6a:7d:74:fc"}}"#;
    let composed = Request {
      id: "00000000-0000-0000-0000-000000000000".try_into().unwrap(),
      jsonrpc: "2.0".to_string(),
      method: Method::ClientGetStatus {
        params: client::GetStatusParams {
          id: "00:21:6a:7d:74:fc".to_string(),
        },
      },
    };

    assert_eq!(serde_json::to_string(&composed).unwrap(), message);
  }

  #[test]
  fn deserialize_client_get_status() {
    let map = DashMap::from_iter([(
      "00000000-0000-0000-0000-000000000000".try_into().unwrap(),
      RequestMethod::ClientGetStatus,
    )]);

    let message = r#"{"id":"00000000-0000-0000-0000-000000000000","jsonrpc":"2.0","result":{"client":{"config":{"instance":1,"latency":0,"name":"","volume":{"muted":false,"percent":74}},"connected":true,"host":{"arch":"x86_64","ip":"127.0.0.1","mac":"00:21:6a:7d:74:fc","name":"T400","os":"Linux Mint 17.3 Rosa"},"id":"00:21:6a:7d:74:fc","lastSeen":{"sec":1488026416,"usec":135973},"snapclient":{"name":"Snapclient","protocolVersion":2,"version":"0.10.0"}}}}"#;
    let snapcast_message = SnapcastDeserializer::de(message, &map).unwrap();

    assert_eq!(
      snapcast_message,
      Message::Result {
        id: "00000000-0000-0000-0000-000000000000".try_into().unwrap(),
        jsonrpc: "2.0".to_string(),
        result: Box::new(SnapcastResult::ClientGetStatus(client::GetStatusResult {
          client: client::Client {
            id: "00:21:6a:7d:74:fc".to_string(),
            connected: true,
            config: client::ClientConfig {
              instance: 1,
              latency: 0,
              name: "".to_string(),
              volume: client::ClientVolume {
                muted: false,
                percent: 74
              }
            },
            host: client::Host {
              arch: "x86_64".to_string(),
              ip: "127.0.0.1".to_string(),
              mac: "00:21:6a:7d:74:fc".to_string(),
              name: "T400".to_string(),
              os: "Linux Mint 17.3 Rosa".to_string()
            },
            last_seen: client::LastSeen {
              sec: 1488026416,
              usec: 135973
            },
            snapclient: client::Snapclient {
              name: "Snapclient".to_string(),
              protocol_version: 2,
              version: "0.10.0".to_string()
            }
          }
        }))
      }
    );
  }

  #[test]
  fn serialize_group_get_status() {
    let message = r#"{"id":"00000000-0000-0000-0000-000000000000","jsonrpc":"2.0","method":"Group.GetStatus","params":{"id":"4dcc4e3b-c699-a04b-7f0c-8260d23c43e1"}}"#;
    let composed = Request {
      id: "00000000-0000-0000-0000-000000000000".try_into().unwrap(),
      jsonrpc: "2.0".to_string(),
      method: Method::GroupGetStatus {
        params: group::GetStatusParams {
          id: "4dcc4e3b-c699-a04b-7f0c-8260d23c43e1".to_string(),
        },
      },
    };

    assert_eq!(serde_json::to_string(&composed).unwrap(), message);
  }

  #[test]
  fn deserialize_group_get_status() {
    let map = DashMap::new();

    let message = r#"{"id":"00000000-0000-0000-0000-000000000000","jsonrpc":"2.0","result":{"group":{"clients":[{"config":{"instance":2,"latency":10,"name":"Laptop","volume":{"muted":false,"percent":48}},"connected":true,"host":{"arch":"x86_64","ip":"127.0.0.1","mac":"00:21:6a:7d:74:fc","name":"T400","os":"Linux Mint 17.3 Rosa"},"id":"00:21:6a:7d:74:fc#2","lastSeen":{"sec":1488026485,"usec":644997},"snapclient":{"name":"Snapclient","protocolVersion":2,"version":"0.10.0"}},{"config":{"instance":1,"latency":0,"name":"","volume":{"muted":false,"percent":74}},"connected":true,"host":{"arch":"x86_64","ip":"127.0.0.1","mac":"00:21:6a:7d:74:fc","name":"T400","os":"Linux Mint 17.3 Rosa"},"id":"00:21:6a:7d:74:fc","lastSeen":{"sec":1488026481,"usec":223747},"snapclient":{"name":"Snapclient","protocolVersion":2,"version":"0.10.0"}}],"id":"4dcc4e3b-c699-a04b-7f0c-8260d23c43e1","muted":true,"name":"","stream_id":"stream 1"}}}"#;
    let snapcast_message = SnapcastDeserializer::de(message, &map).unwrap();

    assert_eq!(
      snapcast_message,
      Message::Result {
        id: "00000000-0000-0000-0000-000000000000".try_into().unwrap(),
        jsonrpc: "2.0".to_string(),
        result: Box::new(SnapcastResult::GroupGetStatus(group::GetStatusResult {
          group: group::Group {
            id: "4dcc4e3b-c699-a04b-7f0c-8260d23c43e1".to_string(),
            muted: true,
            name: "".to_string(),
            stream_id: "stream 1".to_string(),
            clients: vec![
              client::Client {
                id: "00:21:6a:7d:74:fc#2".to_string(),
                connected: true,
                config: client::ClientConfig {
                  instance: 2,
                  latency: 10,
                  name: "Laptop".to_string(),
                  volume: client::ClientVolume {
                    muted: false,
                    percent: 48
                  }
                },
                host: client::Host {
                  arch: "x86_64".to_string(),
                  ip: "127.0.0.1".to_string(),
                  mac: "00:21:6a:7d:74:fc".to_string(),
                  name: "T400".to_string(),
                  os: "Linux Mint 17.3 Rosa".to_string()
                },
                last_seen: client::LastSeen {
                  sec: 1488026485,
                  usec: 644997
                },
                snapclient: client::Snapclient {
                  name: "Snapclient".to_string(),
                  protocol_version: 2,
                  version: "0.10.0".to_string()
                }
              },
              client::Client {
                id: "00:21:6a:7d:74:fc".to_string(),
                connected: true,
                config: client::ClientConfig {
                  instance: 1,
                  latency: 0,
                  name: "".to_string(),
                  volume: client::ClientVolume {
                    muted: false,
                    percent: 74
                  }
                },
                host: client::Host {
                  arch: "x86_64".to_string(),
                  ip: "127.0.0.1".to_string(),
                  mac: "00:21:6a:7d:74:fc".to_string(),
                  name: "T400".to_string(),
                  os: "Linux Mint 17.3 Rosa".to_string()
                },
                last_seen: client::LastSeen {
                  sec: 1488026481,
                  usec: 223747
                },
                snapclient: client::Snapclient {
                  name: "Snapclient".to_string(),
                  protocol_version: 2,
                  version: "0.10.0".to_string()
                }
              }
            ]
          }
        }))
      }
    )
  }

  #[test]
  fn serialize_server_get_status() {
    let message = r#"{"id":"00000000-0000-0000-0000-000000000000","jsonrpc":"2.0","method":"Server.GetStatus"}"#;
    let composed = Request {
      id: "00000000-0000-0000-0000-000000000000".try_into().unwrap(),
      jsonrpc: "2.0".to_string(),
      method: Method::ServerGetStatus,
    };

    assert_eq!(serde_json::to_string(&composed).unwrap(), message);
  }

  #[test]
  fn deserialize_server_get_status() {
    let map = DashMap::new();

    let message = r#"{"id":"00000000-0000-0000-0000-000000000000","jsonrpc":"2.0","result":{"server":{"groups":[{"clients":[{"config":{"instance":1,"latency":0,"name":"","volume":{"muted":false,"percent":100}},"connected":true,"host":{"arch":"aarch64","ip":"172.16.3.109","mac":"2c:cf:67:47:cd:4a","name":"porch-musical-pi","os":"Debian GNU/Linux 12 (bookworm)"},"id":"Porches Pi","lastSeen":{"sec":1718314437,"usec":278423},"snapclient":{"name":"Snapclient","protocolVersion":2,"version":"0.28.0"}}],"id":"960ead7d-101a-88e9-1bee-b1c5f25efa9f","muted":false,"name":"","stream_id":"Porches Spotify"},{"clients":[{"config":{"instance":1,"latency":0,"name":"","volume":{"muted":false,"percent":100}},"connected":true,"host":{"arch":"aarch64","ip":"172.16.2.171","mac":"d8:3a:dd:80:a0:87","name":"family-musical-pi","os":"Debian GNU/Linux 12 (bookworm)"},"id":"Family Pi","lastSeen":{"sec":1718314437,"usec":461576},"snapclient":{"name":"Snapclient","protocolVersion":2,"version":"0.28.0"}}],"id":"22a54ef3-54f6-949b-2eed-2ad83d1dab56","muted":false,"name":"","stream_id":"Kitchen Spotify"},{"clients":[{"config":{"instance":1,"latency":0,"name":"","volume":{"muted":false,"percent":100}},"connected":true,"host":{"arch":"aarch64","ip":"172.16.3.38","mac":"2c:cf:67:47:cd:03","name":"bonus-musical-pi","os":"Debian GNU/Linux 12 (bookworm)"},"id":"Bonus Pi","lastSeen":{"sec":1718060095,"usec":922290},"snapclient":{"name":"Snapclient","protocolVersion":2,"version":"0.28.0"}}],"id":"a67bfc41-9286-48b9-a48c-383fcc16070f","muted":false,"name":"","stream_id":"Porches Spotify"},{"clients":[{"config":{"instance":1,"latency":0,"name":"","volume":{"muted":false,"percent":100}},"connected":false,"host":{"arch":"aarch64","ip":"172.16.2.242","mac":"2c:cf:67:47:ca:ca","name":"bonus-sub-musical-pi","os":"Debian GNU/Linux 12 (bookworm)"},"id":"Bonus Sub Pi","lastSeen":{"sec":1718062516,"usec":632403},"snapclient":{"name":"Snapclient","protocolVersion":2,"version":"0.28.0"}}],"id":"46a2b853-5f6e-37a1-00e0-445c98e5826a","muted":false,"name":"","stream_id":"Porches Spotify"},{"clients":[{"config":{"instance":1,"latency":0,"name":"","volume":{"muted":false,"percent":100}},"connected":true,"host":{"arch":"aarch64","ip":"172.16.2.240","mac":"d8:3a:dd:80:a0:cc","name":"family-sub-musical-pi","os":"Debian GNU/Linux 12 (bookworm)"},"id":"Family Sub Pi","lastSeen":{"sec":1718314437,"usec":344666},"snapclient":{"name":"Snapclient","protocolVersion":2,"version":"0.28.0"}}],"id":"28025fcd-1435-67f1-6fed-eb5117aa436c","muted":false,"name":"","stream_id":"Kitchen Spotify"},{"clients":[{"config":{"instance":1,"latency":0,"name":"","volume":{"muted":false,"percent":100}},"connected":true,"host":{"arch":"armv6l","ip":"172.16.1.56","mac":"b8:27:eb:62:a0:01","name":"joey-room-musical-pi","os":"Raspbian GNU/Linux 12 (bookworm)"},"id":"Joey Room Pi","lastSeen":{"sec":1718314437,"usec":51860},"snapclient":{"name":"Snapclient","protocolVersion":2,"version":"0.28.0"}}],"id":"47d70477-d74d-38e1-b949-7a637b34ee27","muted":false,"name":"","stream_id":"Joey Room Spotify"}],"server":{"host":{"arch":"x86_64","ip":"","mac":"","name":"9960edc046a3","os":"Alpine Linux v3.19"},"snapserver":{"controlProtocolVersion":1,"name":"Snapserver","protocolVersion":1,"version":"0.28.0"}},"streams":[{"id":"Porches Spotify","properties":{"canControl":false,"canGoNext":false,"canGoPrevious":false,"canPause":false,"canPlay":false,"canSeek":false,"metadata":{"artData":{"data":"PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iVVRGLTgiIHN0YW5kYWxvbmU9Im5vIj8+CjxzdmcgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIiBoZWlnaHQ9IjE2OHB4IiB3aWR0aD0iMTY4cHgiIHZlcnNpb249IjEuMSIgdmlld0JveD0iMCAwIDE2OCAxNjgiPgogPHBhdGggZmlsbD0iIzFFRDc2MCIgZD0ibTgzLjk5NiAwLjI3N2MtNDYuMjQ5IDAtODMuNzQzIDM3LjQ5My04My43NDMgODMuNzQyIDAgNDYuMjUxIDM3LjQ5NCA4My43NDEgODMuNzQzIDgzLjc0MSA0Ni4yNTQgMCA4My43NDQtMzcuNDkgODMuNzQ0LTgzLjc0MSAwLTQ2LjI0Ni0zNy40OS04My43MzgtODMuNzQ1LTgzLjczOGwwLjAwMS0wLjAwNHptMzguNDA0IDEyMC43OGMtMS41IDIuNDYtNC43MiAzLjI0LTcuMTggMS43My0xOS42NjItMTIuMDEtNDQuNDE0LTE0LjczLTczLjU2NC04LjA3LTIuODA5IDAuNjQtNS42MDktMS4xMi02LjI0OS0zLjkzLTAuNjQzLTIuODEgMS4xMS01LjYxIDMuOTI2LTYuMjUgMzEuOS03LjI5MSA1OS4yNjMtNC4xNSA4MS4zMzcgOS4zNCAyLjQ2IDEuNTEgMy4yNCA0LjcyIDEuNzMgNy4xOHptMTAuMjUtMjIuODA1Yy0xLjg5IDMuMDc1LTUuOTEgNC4wNDUtOC45OCAyLjE1NS0yMi41MS0xMy44MzktNTYuODIzLTE3Ljg0Ni04My40NDgtOS43NjQtMy40NTMgMS4wNDMtNy4xLTAuOTAzLTguMTQ4LTQuMzUtMS4wNC0zLjQ1MyAwLjkwNy03LjA5MyA0LjM1NC04LjE0MyAzMC40MTMtOS4yMjggNjguMjIyLTQuNzU4IDk0LjA3MiAxMS4xMjcgMy4wNyAxLjg5IDQuMDQgNS45MSAyLjE1IDguOTc2di0wLjAwMXptMC44OC0yMy43NDRjLTI2Ljk5LTE2LjAzMS03MS41Mi0xNy41MDUtOTcuMjg5LTkuNjg0LTQuMTM4IDEuMjU1LTguNTE0LTEuMDgxLTkuNzY4LTUuMjE5LTEuMjU0LTQuMTQgMS4wOC04LjUxMyA1LjIyMS05Ljc3MSAyOS41ODEtOC45OCA3OC43NTYtNy4yNDUgMTA5LjgzIDExLjIwMiAzLjczIDIuMjA5IDQuOTUgNy4wMTYgMi43NCAxMC43MzMtMi4yIDMuNzIyLTcuMDIgNC45NDktMTAuNzMgMi43Mzl6Ii8+Cjwvc3ZnPgo=","extension":"svg"},"artUrl":"http://9960edc046a3:1780/__image_cache?name=cd91d51d70227e57d35950777b3d1aac.svg","duration":217.94500732421875,"title":"leave in five"}},"status":"idle","uri":{"fragment":"","host":"","path":"/usr/bin/librespot","query":{"autoplay":"true","bitrate":"320","chunk_ms":"20","codec":"flac","devicename":"Porches","name":"Porches Spotify","sampleformat":"44100:16:2","volume":"50"},"raw":"librespot:////usr/bin/librespot?autoplay=true&bitrate=320&chunk_ms=20&codec=flac&devicename=Porches&name=Porches Spotify&sampleformat=44100:16:2&volume=50","scheme":"librespot"}},{"id":"Kitchen Spotify","properties":{"canControl":false,"canGoNext":false,"canGoPrevious":false,"canPause":false,"canPlay":false,"canSeek":false,"metadata":{"artData":{"data":"PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iVVRGLTgiIHN0YW5kYWxvbmU9Im5vIj8+CjxzdmcgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIiBoZWlnaHQ9IjE2OHB4IiB3aWR0aD0iMTY4cHgiIHZlcnNpb249IjEuMSIgdmlld0JveD0iMCAwIDE2OCAxNjgiPgogPHBhdGggZmlsbD0iIzFFRDc2MCIgZD0ibTgzLjk5NiAwLjI3N2MtNDYuMjQ5IDAtODMuNzQzIDM3LjQ5My04My43NDMgODMuNzQyIDAgNDYuMjUxIDM3LjQ5NCA4My43NDEgODMuNzQzIDgzLjc0MSA0Ni4yNTQgMCA4My43NDQtMzcuNDkgODMuNzQ0LTgzLjc0MSAwLTQ2LjI0Ni0zNy40OS04My43MzgtODMuNzQ1LTgzLjczOGwwLjAwMS0wLjAwNHptMzguNDA0IDEyMC43OGMtMS41IDIuNDYtNC43MiAzLjI0LTcuMTggMS43My0xOS42NjItMTIuMDEtNDQuNDE0LTE0LjczLTczLjU2NC04LjA3LTIuODA5IDAuNjQtNS42MDktMS4xMi02LjI0OS0zLjkzLTAuNjQzLTIuODEgMS4xMS01LjYxIDMuOTI2LTYuMjUgMzEuOS03LjI5MSA1OS4yNjMtNC4xNSA4MS4zMzcgOS4zNCAyLjQ2IDEuNTEgMy4yNCA0LjcyIDEuNzMgNy4xOHptMTAuMjUtMjIuODA1Yy0xLjg5IDMuMDc1LTUuOTEgNC4wNDUtOC45OCAyLjE1NS0yMi41MS0xMy44MzktNTYuODIzLTE3Ljg0Ni04My40NDgtOS43NjQtMy40NTMgMS4wNDMtNy4xLTAuOTAzLTguMTQ4LTQuMzUtMS4wNC0zLjQ1MyAwLjkwNy03LjA5MyA0LjM1NC04LjE0MyAzMC40MTMtOS4yMjggNjguMjIyLTQuNzU4IDk0LjA3MiAxMS4xMjcgMy4wNyAxLjg5IDQuMDQgNS45MSAyLjE1IDguOTc2di0wLjAwMXptMC44OC0yMy43NDRjLTI2Ljk5LTE2LjAzMS03MS41Mi0xNy41MDUtOTcuMjg5LTkuNjg0LTQuMTM4IDEuMjU1LTguNTE0LTEuMDgxLTkuNzY4LTUuMjE5LTEuMjU0LTQuMTQgMS4wOC04LjUxMyA1LjIyMS05Ljc3MSAyOS41ODEtOC45OCA3OC43NTYtNy4yNDUgMTA5LjgzIDExLjIwMiAzLjczIDIuMjA5IDQuOTUgNy4wMTYgMi43NCAxMC43MzMtMi4yIDMuNzIyLTcuMDIgNC45NDktMTAuNzMgMi43Mzl6Ii8+Cjwvc3ZnPgo=","extension":"svg"},"artUrl":"http://9960edc046a3:1780/__image_cache?name=efc69e1ab3519570d890ee4f551bd908.svg","duration":169.99000549316406,"title":"BLEED"}},"status":"idle","uri":{"fragment":"","host":"","path":"/usr/bin/librespot","query":{"autoplay":"true","bitrate":"320","chunk_ms":"20","codec":"flac","devicename":"Kitchen","name":"Kitchen Spotify","sampleformat":"44100:16:2","volume":"50"},"raw":"librespot:////usr/bin/librespot?autoplay=true&bitrate=320&chunk_ms=20&codec=flac&devicename=Kitchen&name=Kitchen Spotify&sampleformat=44100:16:2&volume=50","scheme":"librespot"}},{"id":"Joey Room Spotify","properties":{"canControl":false,"canGoNext":false,"canGoPrevious":false,"canPause":false,"canPlay":false,"canSeek":false,"metadata":{"artData":{"data":"PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iVVRGLTgiIHN0YW5kYWxvbmU9Im5vIj8+CjxzdmcgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIiBoZWlnaHQ9IjE2OHB4IiB3aWR0aD0iMTY4cHgiIHZlcnNpb249IjEuMSIgdmlld0JveD0iMCAwIDE2OCAxNjgiPgogPHBhdGggZmlsbD0iIzFFRDc2MCIgZD0ibTgzLjk5NiAwLjI3N2MtNDYuMjQ5IDAtODMuNzQzIDM3LjQ5My04My43NDMgODMuNzQyIDAgNDYuMjUxIDM3LjQ5NCA4My43NDEgODMuNzQzIDgzLjc0MSA0Ni4yNTQgMCA4My43NDQtMzcuNDkgODMuNzQ0LTgzLjc0MSAwLTQ2LjI0Ni0zNy40OS04My43MzgtODMuNzQ1LTgzLjczOGwwLjAwMS0wLjAwNHptMzguNDA0IDEyMC43OGMtMS41IDIuNDYtNC43MiAzLjI0LTcuMTggMS43My0xOS42NjItMTIuMDEtNDQuNDE0LTE0LjczLTczLjU2NC04LjA3LTIuODA5IDAuNjQtNS42MDktMS4xMi02LjI0OS0zLjkzLTAuNjQzLTIuODEgMS4xMS01LjYxIDMuOTI2LTYuMjUgMzEuOS03LjI5MSA1OS4yNjMtNC4xNSA4MS4zMzcgOS4zNCAyLjQ2IDEuNTEgMy4yNCA0LjcyIDEuNzMgNy4xOHptMTAuMjUtMjIuODA1Yy0xLjg5IDMuMDc1LTUuOTEgNC4wNDUtOC45OCAyLjE1NS0yMi41MS0xMy44MzktNTYuODIzLTE3Ljg0Ni04My40NDgtOS43NjQtMy40NTMgMS4wNDMtNy4xLTAuOTAzLTguMTQ4LTQuMzUtMS4wNC0zLjQ1MyAwLjkwNy03LjA5MyA0LjM1NC04LjE0MyAzMC40MTMtOS4yMjggNjguMjIyLTQuNzU4IDk0LjA3MiAxMS4xMjcgMy4wNyAxLjg5IDQuMDQgNS45MSAyLjE1IDguOTc2di0wLjAwMXptMC44OC0yMy43NDRjLTI2Ljk5LTE2LjAzMS03MS41Mi0xNy41MDUtOTcuMjg5LTkuNjg0LTQuMTM4IDEuMjU1LTguNTE0LTEuMDgxLTkuNzY4LTUuMjE5LTEuMjU0LTQuMTQgMS4wOC04LjUxMyA1LjIyMS05Ljc3MSAyOS41ODEtOC45OCA3OC43NTYtNy4yNDUgMTA5LjgzIDExLjIwMiAzLjczIDIuMjA5IDQuOTUgNy4wMTYgMi43NCAxMC43MzMtMi4yIDMuNzIyLTcuMDIgNC45NDktMTAuNzMgMi43Mzl6Ii8+Cjwvc3ZnPgo=","extension":"svg"},"artUrl":"http://9960edc046a3:1780/__image_cache?name=db1b174342c6589a1b1786848c88176d.svg","duration":188.20799255371094,"title":"Endeavor"}},"status":"idle","uri":{"fragment":"","host":"","path":"/usr/bin/librespot","query":{"autoplay":"true","bitrate":"320","chunk_ms":"20","codec":"flac","devicename":"Joey%s Room","name":"Joey Room Spotify","sampleformat":"44100:16:2","volume":"50"},"raw":"librespot:////usr/bin/librespot?autoplay=true&bitrate=320&chunk_ms=20&codec=flac&devicename=Joey%s Room&name=Joey Room Spotify&sampleformat=44100:16:2&volume=50","scheme":"librespot"}}]}}}"#;
    let snapcast_message: Message = SnapcastDeserializer::de(message, &map).unwrap();

    println!("{:?}", snapcast_message);
  }

  #[test]
  fn deserialize_notification() {
    let map = DashMap::new();

    let message = r#"{"jsonrpc":"2.0","method":"Client.OnVolumeChanged","params":{"id":"test","volume":{"muted":false,"percent":50}}}"#;
    let snapcast_message = SnapcastDeserializer::de(message, &map).unwrap();

    assert_eq!(
      snapcast_message,
      Message::Notification {
        jsonrpc: "2.0".to_string(),
        method: Box::new(Notification::ClientOnVolumeChanged {
          params: Box::new(client::OnVolumeChangedParams {
            id: "test".to_string(),
            volume: client::ClientVolume {
              muted: false,
              percent: 50
            }
          })
        })
      }
    );
  }
}
