use serde::{Deserialize, Serialize};

use super::request::RequestMethod;
use crate::{client, group, server, stream};

/// The result of a Snapcast request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SnapcastResult {
  // client
  #[serde(rename = "Client.GetStatus")]
  ClientGetStatus(client::GetStatusResult),
  #[serde(rename = "Client.SetVolume")]
  ClientSetVolume(String, client::SetVolumeResult),
  #[serde(rename = "Client.SetLatency")]
  ClientSetLatency(String, client::SetLatencyResult),
  #[serde(rename = "Client.SetName")]
  ClientSetName(String, client::SetNameResult),

  // group
  #[serde(rename = "Group.GetStatus")]
  GroupGetStatus(group::GetStatusResult),
  #[serde(rename = "Group.SetMute")]
  GroupSetMute(String, group::SetMuteResult),
  #[serde(rename = "Group.SetStream")]
  GroupSetStream(String, group::SetStreamResult),
  #[serde(rename = "Group.SetClients")]
  GroupSetClients(group::SetClientsResult),
  #[serde(rename = "Group.SetName")]
  GroupSetName(String, group::SetNameResult),

  // server
  #[serde(rename = "Server.GetRPCVersion")]
  ServerGetRPCVersion(server::GetRpcVersionResult),
  #[serde(rename = "Server.GetStatus")]
  ServerGetStatus(server::GetStatusResult),
  #[serde(rename = "Server.DeleteClient")]
  ServerDeleteClient(server::DeleteClientResult),

  // stream
  #[serde(rename = "Stream.AddStream")]
  StreamAddStream(stream::AddStreamResult),
  #[serde(rename = "Stream.RemoveStream")]
  StreamRemoveStream(stream::RemoveStreamResult),
  #[serde(rename = "Stream.Control")]
  StreamControl(stream::ControlResult),
  #[serde(rename = "Stream.SetProperty")]
  StreamSetProperty(stream::SetPropertiesResult),
}

impl TryFrom<(RequestMethod, serde_json::Value)> for SnapcastResult {
  type Error = serde_json::Error;

  fn try_from((method, value): (RequestMethod, serde_json::Value)) -> Result<Self, Self::Error> {
    match method {
      // client
      RequestMethod::ClientGetStatus => Ok(SnapcastResult::ClientGetStatus(serde_json::from_value(value)?)),
      RequestMethod::ClientSetVolume(id) => Ok(SnapcastResult::ClientSetVolume(id, serde_json::from_value(value)?)),
      RequestMethod::ClientSetLatency(id) => Ok(SnapcastResult::ClientSetLatency(id, serde_json::from_value(value)?)),
      RequestMethod::ClientSetName(id) => Ok(SnapcastResult::ClientSetName(id, serde_json::from_value(value)?)),

      // group
      RequestMethod::GroupGetStatus => Ok(SnapcastResult::GroupGetStatus(serde_json::from_value(value)?)),
      RequestMethod::GroupSetMute(id) => Ok(SnapcastResult::GroupSetMute(id, serde_json::from_value(value)?)),
      RequestMethod::GroupSetStream(id) => Ok(SnapcastResult::GroupSetStream(id, serde_json::from_value(value)?)),
      RequestMethod::GroupSetClients => Ok(SnapcastResult::GroupSetClients(serde_json::from_value(value)?)),
      RequestMethod::GroupSetName(id) => Ok(SnapcastResult::GroupSetName(id, serde_json::from_value(value)?)),

      // server
      RequestMethod::ServerGetRPCVersion => Ok(SnapcastResult::ServerGetRPCVersion(serde_json::from_value(value)?)),
      RequestMethod::ServerGetStatus => Ok(SnapcastResult::ServerGetStatus(serde_json::from_value(value)?)),
      RequestMethod::ServerDeleteClient => Ok(SnapcastResult::ServerDeleteClient(serde_json::from_value(value)?)),

      // stream
      RequestMethod::StreamAddStream => Ok(SnapcastResult::StreamAddStream(serde_json::from_value(value)?)),
      RequestMethod::StreamRemoveStream => Ok(SnapcastResult::StreamRemoveStream(serde_json::from_value(value)?)),
      RequestMethod::StreamControl => Ok(SnapcastResult::StreamControl(serde_json::from_value(value)?)),
      RequestMethod::StreamSetProperty => Ok(SnapcastResult::StreamSetProperty(serde_json::from_value(value)?)),
    }
  }
}
