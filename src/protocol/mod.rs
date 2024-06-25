//! # protocol
//!
//! This module contains all code for communicating with the Snapcast server.
//!
//! The module is split into submodules for each type of request and response for server,
//! as well as a `super` scoped `de` module for deserialization.
//!
//! Most users will only need to interact with the following modules:
//! - [client] for interacting with clients
//! - [group] for interacting with groups
//! - [server] for interacting with the server
//! - [stream] for interacting with streams
//!
//! The [errors] module contains all error types that can be returned from the server. This is
//! reexported higher up in the crate.

use serde::Serialize;

/// module for interacting with client devices connected to the Snapcast server
pub mod client;
/// module for interacting with groups of clients
pub mod group;
/// module for interacting with the Snapcast server itself
pub mod server;
/// module for interacting with audio streams
pub mod stream;

/// module for all error types that can be returned from the server
pub mod errors;

mod de;
mod notification;
mod request;
mod result;

pub use de::DeserializationError;
pub(super) use de::SentRequests;
pub(super) use request::{Request, RequestMethod};

pub use notification::Notification;
pub use request::Method;
pub use result::SnapcastResult;

/// A message received from the Snapcast server
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Message {
  /// A message that is in response to a request
  Result {
    /// The id of the request
    id: uuid::Uuid,
    /// The jsonrpc version (2.0)
    jsonrpc: String,
    /// The result of the request
    result: Box<SnapcastResult>,
  },
  /// An error from the server
  Error {
    /// The id of the request
    id: uuid::Uuid,
    /// The jsonrpc version (2.0)
    jsonrpc: String,
    /// The error
    error: errors::SnapcastError,
  },
  /// A notification from the server
  Notification {
    /// The jsonrpc version (2.0)
    jsonrpc: String,
    /// The notification data itself as a tagged enum
    #[serde(flatten)]
    method: Box<Notification>,
  },
}

/// A message received from the Snapcast server that is not an error
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ValidMessage {
  /// A message that is in response to a request
  Result {
    /// The id of the request
    id: uuid::Uuid,
    /// The jsonrpc version (2.0)
    jsonrpc: String,
    /// The result of the request
    result: Box<SnapcastResult>,
  },
  /// A notification from the server
  Notification {
    /// The jsonrpc version (2.0)
    jsonrpc: String,
    /// The notification data itself as a tagged enum
    #[serde(flatten)]
    method: Box<Notification>,
  },
}

impl TryFrom<Message> for ValidMessage {
  type Error = errors::SnapcastError;

  fn try_from(value: Message) -> Result<Self, Self::Error> {
    match value {
      Message::Result { id, jsonrpc, result } => Ok(ValidMessage::Result { id, jsonrpc, result }),
      Message::Error { error, .. } => Err(error),
      Message::Notification { jsonrpc, method } => Ok(ValidMessage::Notification { jsonrpc, method }),
    }
  }
}
