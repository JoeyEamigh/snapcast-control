use stubborn_io::StubbornTcpStream;
use uuid::Uuid;

use crate::{
  errors,
  protocol::{self, client, group, server, stream, Request, RequestMethod, SentRequests},
  state::WrappedState,
  Message, Method, ValidMessage,
};

type Sender =
  futures::stream::SplitSink<tokio_util::codec::Framed<StubbornTcpStream<std::net::SocketAddr>, Communication>, Method>;
type Receiver =
  futures::stream::SplitStream<tokio_util::codec::Framed<StubbornTcpStream<std::net::SocketAddr>, Communication>>;

/// Struct representing a connection to a Snapcast server.
/// Contains the current state of the server and methods to interact with it.
///
/// call `SnapcastConnection::open` to create a new connection.
pub struct SnapcastConnection {
  /// The current state of the server. The state is Send + Sync, so it can be shared between threads.
  pub state: WrappedState,

  // internal
  sender: Sender,
  receiver: Receiver,
}

impl SnapcastConnection {
  /// open a new connection to a Snapcast server
  ///
  /// # args
  /// `address`: [std::net::SocketAddr] - the address of the Snapcast server
  ///
  /// # returns
  /// a new [SnapcastConnection] struct
  ///
  /// # example
  /// ```no_run
  /// let mut client = SnapcastConnection::open("127.0.0.1:1705".parse().expect("could not parse socket address")).await;
  /// ```
  pub async fn open(address: std::net::SocketAddr) -> Self {
    let state = WrappedState::default();
    let (sender, receiver) = Communication::init(address).await;

    Self {
      state,
      sender,
      receiver,
    }
  }

  /// send a raw command to the Snapcast server
  ///
  /// # args
  /// `command`: [Method] - the command to send
  ///
  /// # returns
  /// an empty [Ok] if the command was sent successfully, or a [ClientError] if there was an error
  ///
  /// # example
  /// ```no_run
  /// client.send(Method::ServerGetStatus).await.expect("could not send command");
  /// ```
  pub async fn send(&mut self, command: Method) -> Result<(), ClientError> {
    use futures::SinkExt;

    self.sender.send(command).await
  }

  /// receive a message from the Snapcast server
  ///
  /// uses a [futures::stream::Next] under the hood, so: \
  /// creates a future that resolves to the next item in the stream
  ///
  /// # returns
  /// an [Option] containing an [Ok] with a [ValidMessage] if a message was received, \
  /// an [Option] containing an [Err] with a [ClientError] if there was an error, \
  /// or [None] if the stream has ended
  ///
  /// # example
  /// ```no_run
  /// let message = client.recv().await.expect("could not receive message");
  /// ```
  pub async fn recv(&mut self) -> Option<Result<ValidMessage, ClientError>> {
    use futures::StreamExt;

    let message = self.receiver.next().await;

    if let Some(Ok(message)) = message {
      match &message {
        Message::Error { error, .. } => return Some(Err(error.clone().into())),
        Message::Result { result, .. } => self.state.handle_result(*result.clone()),
        Message::Notification { method, .. } => self.state.handle_notification(*method.clone()),
      };

      Some(Ok(
        message
          .try_into()
          .expect("this should never fail bc error has returned already"),
      ))
    } else if let Some(Err(err)) = message {
      Some(Err(err))
    } else {
      None
    }
  }

  // client methods
  /// request the current status of a client from the Snapcast server
  ///
  /// wrapper for sending a [ClientGetStatus](Method::ClientGetStatus) command
  ///
  /// # args
  /// `id`: [String] - the id of the client
  ///
  /// # returns
  /// an empty [Ok] if the command was sent successfully, or a [ClientError] if there was an error
  ///
  /// # example
  /// ```no_run
  /// client.client_get_status("client_id".to_string()).await.expect("could not get client status");
  /// ```
  pub async fn client_get_status(&mut self, id: String) -> Result<(), ClientError> {
    self
      .send(Method::ClientGetStatus {
        params: client::GetStatusParams { id },
      })
      .await
  }

  /// set the volume and mute status of a client
  ///
  /// wrapper for sending a [ClientSetVolume](Method::ClientSetVolume) command
  ///
  /// # args
  /// `id`: [String] - the id of the client
  /// `volume`: [client::ClientVolume] - the volume and mute status to set
  ///
  /// # returns
  /// an empty [Ok] if the command was sent successfully, or a [ClientError] if there was an error
  ///
  /// # example
  /// ```no_run
  /// client.client_set_mute("client_id".to_string(), client::ClientVolume { mute: false, volume: 50 }).await.expect("could not set client mute");
  /// ```
  pub async fn client_set_volume(&mut self, id: String, volume: client::ClientVolume) -> Result<(), ClientError> {
    self
      .send(Method::ClientSetVolume {
        params: client::SetVolumeParams { id, volume },
      })
      .await
  }

  /// set the latency of a client
  ///
  /// wrapper for sending a [ClientSetLatency](Method::ClientSetLatency) command
  ///
  /// # args
  /// `id`: [String] - the id of the client
  /// `latency`: [usize] - the latency to set
  ///
  /// # returns
  /// an empty [Ok] if the command was sent successfully, or a [ClientError] if there was an error
  ///
  /// # example
  /// ```no_run
  /// client.client_set_latency("client_id".to_string(), 100).await.expect("could not set client latency");
  /// ```
  pub async fn client_set_latency(&mut self, id: String, latency: usize) -> Result<(), ClientError> {
    self
      .send(Method::ClientSetLatency {
        params: client::SetLatencyParams { id, latency },
      })
      .await
  }

  /// set the name of a client
  ///
  /// wrapper for sending a [ClientSetName](Method::ClientSetName) command
  ///
  /// # args
  /// `id`: [String] - the id of the client
  /// `name`: [String] - the name to set
  ///
  /// # returns
  /// an empty [Ok] if the command was sent successfully, or a [ClientError] if there was an error
  ///
  /// # example
  /// ```no_run
  /// client.client_set_name("client_id".to_string(), "new_name".to_string()).await.expect("could not set client name");
  /// ```
  pub async fn client_set_name(&mut self, id: String, name: String) -> Result<(), ClientError> {
    self
      .send(Method::ClientSetName {
        params: client::SetNameParams { id, name },
      })
      .await
  }

  // group methods
  /// request the current status of a group from the Snapcast server
  ///
  /// wrapper for sending a [GroupGetStatus](Method::GroupGetStatus) command
  ///
  /// # args
  /// `id`: [String] - the id of the group
  ///
  /// # returns
  /// an empty [Ok] if the command was sent successfully, or a [ClientError] if there was an error
  ///
  /// # example
  /// ```no_run
  /// client.group_get_status("group_id".to_string()).await.expect("could not get group status");
  /// ```
  pub async fn group_get_status(&mut self, id: String) -> Result<(), ClientError> {
    self
      .send(Method::GroupGetStatus {
        params: group::GetStatusParams { id },
      })
      .await
  }

  /// set the mute status of a group
  ///
  /// wrapper for sending a [GroupSetMute](Method::GroupSetMute) command
  ///
  /// # args
  /// `id`: [String] - the id of the group
  /// `mute`: [bool] - the mute status to set
  ///
  /// # returns
  /// an empty [Ok] if the command was sent successfully, or a [ClientError] if there was an error
  ///
  /// # example
  /// ```no_run
  /// client.group_set_mute("group_id".to_string(), true).await.expect("could not set group mute");
  /// ```
  pub async fn group_set_mute(&mut self, id: String, mute: bool) -> Result<(), ClientError> {
    self
      .send(Method::GroupSetMute {
        params: group::SetMuteParams { id, mute },
      })
      .await
  }

  /// set the stream of a group
  ///
  /// wrapper for sending a [GroupSetStream](Method::GroupSetStream) command
  ///
  /// # args
  /// `id`: [String] - the id of the group
  /// `stream_id`: [String] - the id of the stream to set
  ///
  /// # returns
  /// an empty [Ok] if the command was sent successfully, or a [ClientError] if there was an error
  ///
  /// # example
  /// ```no_run
  /// client.group_set_stream("group_id".to_string(), "stream_id".to_string()).await.expect("could not set group stream");
  /// ```
  pub async fn group_set_stream(&mut self, id: String, stream_id: String) -> Result<(), ClientError> {
    self
      .send(Method::GroupSetStream {
        params: group::SetStreamParams { id, stream_id },
      })
      .await
  }

  /// set the clients of a group
  ///
  /// wrapper for sending a [GroupSetClients](Method::GroupSetClients) command
  ///
  /// # args
  /// `id`: [String] - the id of the group
  /// `clients`: [Vec]<[String]> - the ids of the clients to set
  ///
  /// # returns
  /// an empty [Ok] if the command was sent successfully, or a [ClientError] if there was an error
  ///
  /// # example
  /// ```no_run
  /// client.group_set_clients("group_id".to_string(), vec!["client_id".to_string()]).await.expect("could not set group clients");
  /// ```
  pub async fn group_set_clients(&mut self, id: String, clients: Vec<String>) -> Result<(), ClientError> {
    self
      .send(Method::GroupSetClients {
        params: group::SetClientsParams { id, clients },
      })
      .await
  }

  /// set the name of a group
  ///
  /// wrapper for sending a [GroupSetName](Method::GroupSetName) command
  ///
  /// # args
  /// `id`: [String] - the id of the group
  /// `name`: [String] - the name to set
  ///
  /// # returns
  /// an empty [Ok] if the command was sent successfully, or a [ClientError] if there was an error
  ///
  /// # example
  /// ```no_run
  /// client.group_set_name("group_id".to_string(), "new_name".to_string()).await.expect("could not set group name");
  /// ```
  pub async fn group_set_name(&mut self, id: String, name: String) -> Result<(), ClientError> {
    self
      .send(Method::GroupSetName {
        params: group::SetNameParams { id, name },
      })
      .await
  }

  // server methods
  /// request the rpc version of the Snapcast server
  ///
  /// wrapper for sending a [ServerGetStatus](Method::ServerGetStatus) command
  ///
  /// # returns
  /// an empty [Ok] if the command was sent successfully, or a [ClientError] if there was an error
  ///
  /// # example
  /// ```no_run
  /// client.server_get_rpc_version().await.expect("could not get server rpc version");
  /// ```
  pub async fn server_get_rpc_version(&mut self) -> Result<(), ClientError> {
    self.send(Method::ServerGetRPCVersion).await
  }

  /// request the current status of the Snapcast server, this is a full refresh for state
  ///
  /// wrapper for sending a [ServerGetStatus](Method::ServerGetStatus) command
  ///
  /// # returns
  /// an empty [Ok] if the command was sent successfully, or a [ClientError] if there was an error
  ///
  /// # example
  /// ```no_run
  /// client.server_get_status().await.expect("could not get server status");
  /// ```
  pub async fn server_get_status(&mut self) -> Result<(), ClientError> {
    self.send(Method::ServerGetStatus).await
  }

  /// forcefully delete a client from the Snapcast server
  ///
  /// wrapper for sending a [ServerDeleteClient](Method::ServerDeleteClient) command
  ///
  /// # args
  /// `id`: [String] - the id of the client to delete
  ///
  /// # returns
  /// an empty [Ok] if the command was sent successfully, or a [ClientError] if there was an error
  ///
  /// # example
  /// ```no_run
  /// client.server_delete_client("client_id".to_string()).await.expect("could not delete client");
  /// ```
  pub async fn server_delete_client(&mut self, id: String) -> Result<(), ClientError> {
    self
      .send(Method::ServerDeleteClient {
        params: server::DeleteClientParams { id },
      })
      .await
  }

  // stream methods
  /// add a new stream to the Snapcast server
  ///
  /// wrapper for sending a [StreamAddStream](Method::StreamAddStream) command
  ///
  /// # args
  /// `stream_uri`: [String] - the uri of the stream to add
  ///
  /// # returns
  /// an empty [Ok] if the command was sent successfully, or a [ClientError] if there was an error
  ///
  /// # example
  /// ```no_run
  /// client.stream_add_stream("librespot:///usr/bin/librespot?name=Spotify&...".to_string()).await.expect("could not add stream");
  /// ```
  pub async fn stream_add_stream(&mut self, stream_uri: String) -> Result<(), ClientError> {
    self
      .send(Method::StreamAddStream {
        params: stream::AddStreamParams { stream_uri },
      })
      .await
  }

  /// remove a stream from the Snapcast server
  ///
  /// wrapper for sending a [StreamRemoveStream](Method::StreamRemoveStream) command
  ///
  /// # args
  /// `id`: [String] - the id of the stream to remove
  ///
  /// # returns
  /// an empty [Ok] if the command was sent successfully, or a [ClientError] if there was an error
  ///
  /// # example
  /// ```no_run
  /// client.stream_remove_stream("stream_id".to_string()).await.expect("could not remove stream");
  /// ```
  pub async fn stream_remove_stream(&mut self, id: String) -> Result<(), ClientError> {
    self
      .send(Method::StreamRemoveStream {
        params: stream::RemoveStreamParams { id },
      })
      .await
  }

  /// control a stream on the Snapcast server
  ///
  /// wrapper for sending a [StreamControl](Method::StreamControl) command
  ///
  /// # args
  /// `id`: [String] - the id of the stream to control
  /// `command`: [stream::ControlCommand] - the command to send to the stream
  ///
  /// # returns
  /// an empty [Ok] if the command was sent successfully, or a [ClientError] if there was an error
  ///
  /// # example
  /// ```no_run
  /// client.stream_control("stream_id".to_string(), stream::ControlCommand::Pause).await.expect("could not control stream");
  /// ```
  pub async fn stream_control(&mut self, id: String, command: stream::ControlCommand) -> Result<(), ClientError> {
    self
      .send(Method::StreamControl {
        params: stream::ControlParams { id, command },
      })
      .await
  }

  /// set the property of a stream on the Snapcast server
  ///
  /// wrapper for sending a [StreamSetProperty](Method::StreamSetProperty) command
  ///
  /// # args
  /// `id`: [String] - the id of the stream to control
  /// `properties`: [stream::SetPropertyProperties] - the properties to set on the stream
  ///
  /// # returns
  /// an empty [Ok] if the command was sent successfully, or a [ClientError] if there was an error
  ///
  /// # example
  /// ```no_run
  /// client.stream_set_property("stream_id".to_string(), stream::SetPropertyProperties::Shuffle(true)).await.expect("could not set stream property");
  /// ```
  pub async fn stream_set_property(
    &mut self,
    id: String,
    properties: stream::SetPropertyProperties,
  ) -> Result<(), ClientError> {
    self
      .send(Method::StreamSetProperty {
        params: stream::SetPropertyParams { id, properties },
      })
      .await
  }
}

#[derive(Debug, Clone, Default)]
struct Communication {
  purgatory: SentRequests,
}

impl Communication {
  async fn init(address: std::net::SocketAddr) -> (Sender, Receiver) {
    use futures::stream::StreamExt;
    use tokio_util::codec::Decoder;

    let client = Self::default();

    tracing::info!("connecting to snapcast server at {}", address);
    let stream = StubbornTcpStream::connect(address).await.unwrap();
    let (writer, reader) = client.framed(stream).split();

    (writer, reader)
  }
}

impl tokio_util::codec::Decoder for Communication {
  type Item = Message;
  type Error = ClientError;

  fn decode(&mut self, src: &mut tokio_util::bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    use tokio_util::bytes::Buf;

    if src.is_empty() {
      return Ok(None);
    }

    // tracing::trace!("decoding: {:?}", src);

    let lf_pos = src.as_ref().iter().position(|b| *b == b'\n');
    if let Some(lf_pos) = lf_pos {
      let data = src.split_to(lf_pos);
      src.advance(1);

      tracing::debug!("received complete message with length: {}", data.len());
      let message = std::str::from_utf8(&data).unwrap();
      tracing::trace!("completed json message: {:?}", message);

      let message = Message::try_from((message, &self.purgatory))?;
      tracing::trace!("completed deserialized message: {:?}", message);

      return Ok(Some(message));
    }

    Ok(None)
  }
}

impl tokio_util::codec::Encoder<Method> for Communication {
  type Error = ClientError;

  fn encode(&mut self, method: Method, dst: &mut tokio_util::bytes::BytesMut) -> Result<(), Self::Error> {
    tracing::trace!("encoding: {:?}", method);

    let id = Uuid::new_v4();
    let command: RequestMethod = (&method).into();
    tracing::debug!("sending command: {:?}", command);
    self.purgatory.insert(id, command);

    let data = Request {
      id,
      jsonrpc: "2.0".to_string(),
      method,
    };

    let string: String = data.try_into()?;
    let string = format!("{}\n", string);
    tracing::trace!("sending: {:?}", string);

    dst.extend_from_slice(string.as_bytes());

    Ok(())
  }
}

/// Error type for the Snapcast client
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
  /// An error returned by the Snapcast server
  #[error("Snapcast error: {0}")]
  Snapcast(#[from] errors::SnapcastError),
  /// An error communicating with the Snapcast server
  #[error("Communication error: {0}")]
  Io(#[from] std::io::Error),
  /// An error deserializing a message from the Snapcast server
  #[error("Deserialization error: {0}")]
  Deserialization(#[from] protocol::DeserializationError),
  /// An error deserializing the json from the Snapcast server
  #[error("JSON Deserialization error: {0}")]
  JsonDeserialization(#[from] serde_json::Error),
  /// An unknown error
  #[error("Unknown error: {0}")]
  Unknown(String),
}
