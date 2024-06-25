use serde::{Deserialize, Serialize};

pub use super::client::Host;
use super::{group::Group, stream::Stream};

// the server
/// The struct representing the full state of the Snapcast server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Server {
  pub server: ServerDetails,
  pub groups: Vec<Group>,
  pub streams: Vec<Stream>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerDetails {
  pub host: Host,
  pub snapserver: Snapserver,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snapserver {
  pub name: String,
  #[serde(rename = "protocolVersion")]
  pub protocol_version: usize,
  #[serde(rename = "controlProtocolVersion")]
  pub control_protocol_version: usize,
  pub version: String,
}

// params and results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetRpcVersionResult {
  pub major: usize,
  pub minor: usize,
  pub patch: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetStatusResult {
  pub server: Server,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeleteClientParams {
  pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeleteClientResult {
  pub server: Server,
}

// notifications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnUpdateParams {
  pub server: Server,
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use crate::protocol::{
    client::{Client, ClientConfig, ClientVolume, Host, LastSeen, Snapclient},
    stream::StreamUri,
  };

  use super::*;

  #[test]
  fn serialize_server() {
    let server = Server {
      groups: vec![Group {
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
      }],
      server: ServerDetails {
        host: Host {
          arch: "x86_64".to_string(),
          ip: "".to_string(),
          mac: "".to_string(),
          name: "T400".to_string(),
          os: "Linux Mint 17.3 Rosa".to_string(),
        },
        snapserver: Snapserver {
          control_protocol_version: 1,
          name: "Snapserver".to_string(),
          protocol_version: 1,
          version: "0.10.0".to_string(),
        },
      },
      streams: vec![
        Stream {
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
        },
        Stream {
          id: "stream 2".to_string(),
          status: "idle".into(),
          properties: None,
          uri: StreamUri {
            fragment: "".to_string(),
            host: "".to_string(),
            path: "/tmp/snapfifo".to_string(),
            query: HashMap::new(),
            raw: "pipe:///tmp/snapfifo?name=stream 2".to_string(),
            scheme: "pipe".to_string(),
          },
        },
      ],
    };

    serde_json::to_string(&server).unwrap();
  }

  #[test]
  fn deserialize_server() {
    let json = r#"{"groups":[{"clients":[{"config":{"instance":2,"latency":6,"name":"123 456","volume":{"muted":false,"percent":48}},"connected":true,"host":{"arch":"x86_64","ip":"127.0.0.1","mac":"00:21:6a:7d:74:fc","name":"T400","os":"Linux Mint 17.3 Rosa"},"id":"00:21:6a:7d:74:fc#2","lastSeen":{"sec":1488025901,"usec":864472},"snapclient":{"name":"Snapclient","protocolVersion":2,"version":"0.10.0"}},{"config":{"instance":1,"latency":0,"name":"","volume":{"muted":false,"percent":100}},"connected":true,"host":{"arch":"x86_64","ip":"127.0.0.1","mac":"00:21:6a:7d:74:fc","name":"T400","os":"Linux Mint 17.3 Rosa"},"id":"00:21:6a:7d:74:fc","lastSeen":{"sec":1488025905,"usec":45238},"snapclient":{"name":"Snapclient","protocolVersion":2,"version":"0.10.0"}}],"id":"4dcc4e3b-c699-a04b-7f0c-8260d23c43e1","muted":false,"name":"","stream_id":"stream 2"}],"server":{"host":{"arch":"x86_64","ip":"","mac":"","name":"T400","os":"Linux Mint 17.3 Rosa"},"snapserver":{"controlProtocolVersion":1,"name":"Snapserver","protocolVersion":1,"version":"0.10.0"}},"streams":[{"id":"stream 1","status":"idle","uri":{"fragment":"","host":"","path":"/tmp/snapfifo","query":{"chunk_ms":"20","codec":"flac","name":"stream 1","sampleformat":"48000:16:2"},"raw":"pipe:///tmp/snapfifo?name=stream 1","scheme":"pipe"}},{"id":"stream 2","status":"idle","uri":{"fragment":"","host":"","path":"/tmp/snapfifo","query":{"chunk_ms":"20","codec":"flac","name":"stream 2","sampleformat":"48000:16:2"},"raw":"pipe:///tmp/snapfifo?name=stream 2","scheme":"pipe"}}]}"#;
    let server: Server = serde_json::from_str(json).unwrap();

    assert_eq!(server.server.host.name, "T400");
  }
}
