use serde::{Deserialize, Serialize};

use crate::{client, group, server, stream};

/// The method of a request that the client can call
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "method")]
pub enum Method {
  // client
  #[serde(rename = "Client.GetStatus")]
  ClientGetStatus { params: client::GetStatusParams },
  #[serde(rename = "Client.SetVolume")]
  ClientSetVolume { params: client::SetVolumeParams },
  #[serde(rename = "Client.SetLatency")]
  ClientSetLatency { params: client::SetLatencyParams },
  #[serde(rename = "Client.SetName")]
  ClientSetName { params: client::SetNameParams },

  // group
  #[serde(rename = "Group.GetStatus")]
  GroupGetStatus { params: group::GetStatusParams },
  #[serde(rename = "Group.SetMute")]
  GroupSetMute { params: group::SetMuteParams },
  #[serde(rename = "Group.SetStream")]
  GroupSetStream { params: group::SetStreamParams },
  #[serde(rename = "Group.SetClients")]
  GroupSetClients { params: group::SetClientsParams },
  #[serde(rename = "Group.SetName")]
  GroupSetName { params: group::SetNameParams },

  // server
  #[serde(rename = "Server.GetRPCVersion")]
  ServerGetRPCVersion,
  #[serde(rename = "Server.GetStatus")]
  ServerGetStatus,
  #[serde(rename = "Server.DeleteClient")]
  ServerDeleteClient { params: server::DeleteClientParams },

  // stream
  #[serde(rename = "Stream.AddStream")]
  StreamAddStream { params: stream::AddStreamParams },
  #[serde(rename = "Stream.RemoveStream")]
  StreamRemoveStream { params: stream::RemoveStreamParams },
  #[serde(rename = "Stream.Control")]
  StreamControl { params: stream::ControlParams },
  #[serde(rename = "Stream.SetProperty")]
  StreamSetProperty { params: stream::SetPropertyParams },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
  pub id: uuid::Uuid,
  pub jsonrpc: String,
  #[serde(flatten)]
  pub method: Method,
}

impl TryInto<String> for Request {
  type Error = serde_json::Error;

  fn try_into(self) -> Result<String, Self::Error> {
    serde_json::to_string(&self)
  }
}

#[derive(Clone, Debug)]
pub enum RequestMethod {
  // client
  ClientGetStatus,
  ClientSetVolume(String),
  ClientSetLatency(String),
  ClientSetName(String),

  // group
  GroupGetStatus,
  GroupSetMute(String),
  GroupSetStream(String),
  GroupSetClients,
  GroupSetName(String),

  // server
  ServerGetRPCVersion,
  ServerGetStatus,
  ServerDeleteClient,

  // stream
  StreamAddStream,
  StreamRemoveStream,
  StreamControl,
  StreamSetProperty,
}

impl From<&Method> for RequestMethod {
  fn from(method: &Method) -> Self {
    match method {
      // client
      Method::ClientGetStatus { .. } => Self::ClientGetStatus,
      Method::ClientSetVolume { params } => Self::ClientSetVolume(params.id.clone()),
      Method::ClientSetLatency { params } => Self::ClientSetLatency(params.id.clone()),
      Method::ClientSetName { params } => Self::ClientSetName(params.id.clone()),

      // group
      Method::GroupGetStatus { .. } => Self::GroupGetStatus,
      Method::GroupSetMute { params } => Self::GroupSetMute(params.id.clone()),
      Method::GroupSetStream { params } => Self::GroupSetStream(params.id.clone()),
      Method::GroupSetClients { .. } => Self::GroupSetClients,
      Method::GroupSetName { params } => Self::GroupSetName(params.id.clone()),

      // server
      Method::ServerGetRPCVersion => Self::ServerGetRPCVersion,
      Method::ServerGetStatus => Self::ServerGetStatus,
      Method::ServerDeleteClient { .. } => Self::ServerDeleteClient,

      // stream
      Method::StreamAddStream { .. } => Self::StreamAddStream,
      Method::StreamRemoveStream { .. } => Self::StreamRemoveStream,
      Method::StreamControl { .. } => Self::StreamControl,
      Method::StreamSetProperty { .. } => Self::StreamSetProperty,
    }
  }
}
