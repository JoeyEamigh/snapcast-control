use serde::{Deserialize, Serialize};

// the snapclient
/// A client connected to the Snapcast server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Client {
  pub id: String,
  pub connected: bool,
  pub config: ClientConfig,
  pub host: Host,
  pub snapclient: Snapclient,
  #[serde(rename = "lastSeen")]
  pub last_seen: LastSeen,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Host {
  pub arch: String,
  pub ip: String,
  pub mac: String,
  pub name: String,
  pub os: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientConfig {
  pub instance: usize,
  pub latency: usize,
  pub name: String,
  pub volume: ClientVolume,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientVolume {
  pub muted: bool,
  pub percent: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snapclient {
  pub name: String,
  #[serde(rename = "protocolVersion")]
  pub protocol_version: usize,
  pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LastSeen {
  pub sec: usize,
  pub usec: usize,
}

// params and results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetStatusParams {
  pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetStatusResult {
  pub client: Client,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetVolumeParams {
  pub id: String,
  pub volume: ClientVolume,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetVolumeResult {
  pub volume: ClientVolume,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetLatencyParams {
  pub id: String,
  pub latency: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetLatencyResult {
  pub latency: usize,
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
pub struct OnConnectParams {
  pub id: String,
  pub client: Client,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnDisconnectParams {
  pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnVolumeChangedParams {
  pub id: String,
  pub volume: ClientVolume,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnLatencyChangedParams {
  pub id: String,
  pub latency: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnNameChangedParams {
  pub id: String,
  pub name: String,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn serialize_host() {
    let host = Host {
      arch: "x86_64".to_string(),
      ip: "127.0.0.1".to_string(),
      mac: "00:21:6a:7d:74:fc".to_string(),
      name: "T400".to_string(),
      os: "Linux Mint 17.3 Rosa".to_string(),
    };

    let json = serde_json::to_string(&host).unwrap();
    let expected =
      r#"{"arch":"x86_64","ip":"127.0.0.1","mac":"00:21:6a:7d:74:fc","name":"T400","os":"Linux Mint 17.3 Rosa"}"#;

    assert_eq!(json, expected);
  }

  #[test]
  fn deserialize_host() {
    let json =
      r#"{"arch":"x86_64","ip":"127.0.0.1","mac":"00:21:6a:7d:74:fc","name":"T400","os":"Linux Mint 17.3 Rosa"}"#;
    let host: Host = serde_json::from_str(json).unwrap();

    assert_eq!(host.mac, "00:21:6a:7d:74:fc");
  }

  #[test]
  fn serialize_client() {
    let client = Client {
      id: "00:21:6a:7d:74:fc#2".to_string(),
      connected: true,
      config: ClientConfig {
        instance: 2,
        latency: 6,
        name: "123 456".to_string(),
        volume: ClientVolume {
          muted: false,
          percent: 48,
        },
      },
      host: Host {
        arch: "x86_64".to_string(),
        ip: "127.0.0.1".to_string(),
        mac: "00:21:6a:7d:74:fc".to_string(),
        name: "T400".to_string(),
        os: "Linux Mint 17.3 Rosa".to_string(),
      },
      snapclient: Snapclient {
        name: "Snapclient".to_string(),
        protocol_version: 2,
        version: "0.10.0".to_string(),
      },
      last_seen: LastSeen {
        sec: 1488025901,
        usec: 864472,
      },
    };

    let json = serde_json::to_string(&client).unwrap();
    let expected = r#"{"id":"00:21:6a:7d:74:fc#2","connected":true,"config":{"instance":2,"latency":6,"name":"123 456","volume":{"muted":false,"percent":48}},"host":{"arch":"x86_64","ip":"127.0.0.1","mac":"00:21:6a:7d:74:fc","name":"T400","os":"Linux Mint 17.3 Rosa"},"snapclient":{"name":"Snapclient","protocolVersion":2,"version":"0.10.0"},"lastSeen":{"sec":1488025901,"usec":864472}}"#;

    assert_eq!(json, expected);
  }

  #[test]
  fn deserialize_client() {
    let json = r#"{"config":{"instance":2,"latency":6,"name":"123 456","volume":{"muted":false,"percent":48}},"connected":true,"host":{"arch":"x86_64","ip":"127.0.0.1","mac":"00:21:6a:7d:74:fc","name":"T400","os":"Linux Mint 17.3 Rosa"},"id":"00:21:6a:7d:74:fc#2","lastSeen":{"sec":1488025901,"usec":864472},"snapclient":{"name":"Snapclient","protocolVersion":2,"version":"0.10.0"}}"#;
    let client: Client = serde_json::from_str(json).unwrap();

    assert_eq!(client.id, "00:21:6a:7d:74:fc#2");
  }
}
