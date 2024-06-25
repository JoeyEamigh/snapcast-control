use serde::{Deserialize, Serialize};

use crate::{client, group, server, stream};

/// A notification from the Snapcast server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "method")]
pub enum Notification {
  // client
  #[serde(rename = "Client.OnConnect")]
  ClientOnConnect { params: Box<client::OnConnectParams> },
  #[serde(rename = "Client.OnDisconnect")]
  ClientOnDisconnect { params: Box<client::OnDisconnectParams> },
  #[serde(rename = "Client.OnVolumeChanged")]
  ClientOnVolumeChanged { params: Box<client::OnVolumeChangedParams> },
  #[serde(rename = "Client.OnLatencyChanged")]
  ClientOnLatencyChanged {
    params: Box<client::OnLatencyChangedParams>,
  },
  #[serde(rename = "Client.OnNameChanged")]
  ClientOnNameChanged { params: Box<client::OnNameChangedParams> },

  // group
  #[serde(rename = "Group.OnMute")]
  GroupOnMute { params: Box<group::OnMuteParams> },
  #[serde(rename = "Group.OnStreamChanged")]
  GroupOnStreamChanged { params: Box<group::OnStreamChangedParams> },
  #[serde(rename = "Group.OnNameChanged")]
  GroupOnNameChanged { params: Box<group::OnNameChangedParams> },

  // server
  #[serde(rename = "Server.OnUpdate")]
  ServerOnUpdate { params: Box<server::OnUpdateParams> },

  // stream
  #[serde(rename = "Stream.OnUpdate")]
  StreamOnUpdate { params: Box<stream::OnUpdateParams> },
  #[serde(rename = "Stream.OnProperties")]
  StreamOnProperties { params: Box<stream::OnPropertiesParams> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
  // client
  #[serde(rename = "Client.OnConnect")]
  ClientOnConnect,
  #[serde(rename = "Client.OnDisconnect")]
  ClientOnDisconnect,
  #[serde(rename = "Client.OnVolumeChanged")]
  ClientOnVolumeChanged,
  #[serde(rename = "Client.OnLatencyChanged")]
  ClientOnLatencyChanged,
  #[serde(rename = "Client.OnNameChanged")]
  ClientOnNameChanged,

  // group
  #[serde(rename = "Group.OnMute")]
  GroupOnMute,
  #[serde(rename = "Group.OnStreamChanged")]
  GroupOnStreamChanged,
  #[serde(rename = "Group.OnNameChanged")]
  GroupOnNameChanged,

  // server
  #[serde(rename = "Server.OnUpdate")]
  ServerOnUpdate,

  // stream
  #[serde(rename = "Stream.OnUpdate")]
  StreamOnUpdate,
  #[serde(rename = "Stream.OnProperties")]
  StreamOnProperties,
}

pub(crate) struct NotificationMethodConverter(pub NotificationType, pub serde_json::Value);

impl TryFrom<NotificationMethodConverter> for Notification {
  type Error = serde_json::Error;

  fn try_from(value: NotificationMethodConverter) -> Result<Self, Self::Error> {
    let NotificationMethodConverter(method, params) = value;

    match method {
      // client
      NotificationType::ClientOnConnect => Ok(Notification::ClientOnConnect {
        params: serde_json::from_value(params)?,
      }),
      NotificationType::ClientOnDisconnect => Ok(Notification::ClientOnDisconnect {
        params: serde_json::from_value(params)?,
      }),
      NotificationType::ClientOnVolumeChanged => Ok(Notification::ClientOnVolumeChanged {
        params: serde_json::from_value(params)?,
      }),
      NotificationType::ClientOnLatencyChanged => Ok(Notification::ClientOnLatencyChanged {
        params: serde_json::from_value(params)?,
      }),
      NotificationType::ClientOnNameChanged => Ok(Notification::ClientOnNameChanged {
        params: serde_json::from_value(params)?,
      }),

      // group
      NotificationType::GroupOnMute => Ok(Notification::GroupOnMute {
        params: serde_json::from_value(params)?,
      }),
      NotificationType::GroupOnStreamChanged => Ok(Notification::GroupOnStreamChanged {
        params: serde_json::from_value(params)?,
      }),
      NotificationType::GroupOnNameChanged => Ok(Notification::GroupOnNameChanged {
        params: serde_json::from_value(params)?,
      }),

      // server
      NotificationType::ServerOnUpdate => Ok(Notification::ServerOnUpdate {
        params: serde_json::from_value(params)?,
      }),

      // stream
      NotificationType::StreamOnUpdate => Ok(Notification::StreamOnUpdate {
        params: serde_json::from_value(params)?,
      }),
      NotificationType::StreamOnProperties => Ok(Notification::StreamOnProperties {
        params: serde_json::from_value(params)?,
      }),
    }
  }
}
