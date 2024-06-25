use super::{client::Client, server::Server};
use serde::{Deserialize, Serialize};

// the group
/// A group of clients maintained by the Snapcast server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Group {
  pub id: String,
  pub name: String,
  pub stream_id: String,
  pub muted: bool,
  pub clients: Vec<Client>,
}

// params and results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetStatusParams {
  pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetStatusResult {
  pub group: Group,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetMuteParams {
  pub id: String,
  pub mute: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetMuteResult {
  pub mute: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnMuteParams {
  pub id: String,
  pub mute: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetStreamParams {
  pub id: String,
  pub stream_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetStreamResult {
  pub stream_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetClientsParams {
  pub id: String,
  /// vec of client ids
  pub clients: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetClientsResult {
  pub server: Server,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetNameParams {
  pub id: String,
  pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetNameResult {
  pub name: String,
}

// notifications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnStreamChangedParams {
  pub id: String,
  pub stream_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnNameChangedParams {
  pub id: String,
  pub name: String,
}

#[cfg(test)]
mod tests {
  use crate::protocol::client::{ClientConfig, ClientVolume, Host, LastSeen, Snapclient};

  use super::*;

  #[test]
  fn serialize_group() {
    let group = Group {
      id: "4dcc4e3b-c699-a04b-7f0c-8260d23c43e1".to_string(),
      muted: false,
      name: "".to_string(),
      stream_id: "stream 2".to_string(),
      clients: vec![
        Client {
          config: ClientConfig {
            instance: 1,
            latency: 0,
            name: "".to_string(),
            volume: ClientVolume {
              muted: false,
              percent: 100,
            },
          },
          connected: true,
          host: Host {
            arch: "x86_64".to_string(),
            ip: "127.0.0.1".to_string(),
            mac: "00:21:6a:7d:74:fc".to_string(),
            name: "T400".to_string(),
            os: "Linux Mint 17.3 Rosa".to_string(),
          },
          id: "00:21:6a:7d:74:fc".to_string(),
          last_seen: LastSeen {
            sec: 1488025905,
            usec: 45238,
          },
          snapclient: Snapclient {
            name: "Snapclient".to_string(),
            protocol_version: 2,
            version: "0.10.0".to_string(),
          },
        },
        Client {
          config: ClientConfig {
            instance: 2,
            latency: 6,
            name: "123 456".to_string(),
            volume: ClientVolume {
              muted: false,
              percent: 48,
            },
          },
          connected: true,
          host: Host {
            arch: "x86_64".to_string(),
            ip: "127.0.0.1".to_string(),
            mac: "00:21:6a:7d:74:fc".to_string(),
            name: "T400".to_string(),
            os: "Linux Mint 17.3 Rosa".to_string(),
          },
          id: "00:21:6a:7d:74:fc#2".to_string(),
          last_seen: LastSeen {
            sec: 1488025901,
            usec: 864472,
          },
          snapclient: Snapclient {
            name: "Snapclient".to_string(),
            protocol_version: 2,
            version: "0.10.0".to_string(),
          },
        },
      ],
    };

    serde_json::to_string(&group).unwrap();
  }

  #[test]
  fn deserialize_group() {
    let json = r#"{"clients":[{"config":{"instance":2,"latency":6,"name":"123 456","volume":{"muted":false,"percent":48}},"connected":true,"host":{"arch":"x86_64","ip":"127.0.0.1","mac":"00:21:6a:7d:74:fc","name":"T400","os":"Linux Mint 17.3 Rosa"},"id":"00:21:6a:7d:74:fc#2","lastSeen":{"sec":1488025901,"usec":864472},"snapclient":{"name":"Snapclient","protocolVersion":2,"version":"0.10.0"}},{"config":{"instance":1,"latency":0,"name":"","volume":{"muted":false,"percent":100}},"connected":true,"host":{"arch":"x86_64","ip":"127.0.0.1","mac":"00:21:6a:7d:74:fc","name":"T400","os":"Linux Mint 17.3 Rosa"},"id":"00:21:6a:7d:74:fc","lastSeen":{"sec":1488025905,"usec":45238},"snapclient":{"name":"Snapclient","protocolVersion":2,"version":"0.10.0"}}],"id":"4dcc4e3b-c699-a04b-7f0c-8260d23c43e1","muted":false,"name":"","stream_id":"stream 2"}"#;
    let group: Group = serde_json::from_str(json).unwrap();

    assert_eq!(group.id, "4dcc4e3b-c699-a04b-7f0c-8260d23c43e1");
  }
}
