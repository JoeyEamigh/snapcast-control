use dashmap::{mapref::entry::Entry, DashMap};
use std::{
  cell::OnceCell,
  collections::HashSet,
  sync::{Arc, RwLock},
};

use crate::protocol::{
  client::{Client, ClientVolume},
  group::Group,
  server::{Server, ServerDetails},
  stream::{Stream, StreamProperties},
  Notification, SnapcastResult,
};

/// group details as stored in the state object
///
/// this contains a [HashSet] of client ids instead of a vec of client structs
#[derive(Clone, Debug)]
pub struct StateGroup {
  /// group id
  pub id: String,
  /// group name
  pub name: String,
  /// stream id
  pub stream_id: String,
  /// group muted status
  pub muted: bool,
  /// set of client ids in group
  pub clients: HashSet<String>,
}

/// A wrapped state that can be shared between threads
pub type WrappedState = Arc<State>;

/// The state of the Snapcast server, automatically kept up to date by the client
#[derive(Debug, Default)]
pub struct State {
  /// host and snapserver information
  pub server: OnceCell<RwLock<ServerDetails>>,
  /// group information keyed by group id
  pub groups: DashMap<String, StateGroup>,
  /// client information keyed by client id
  pub clients: DashMap<String, Client>,
  /// stream information keyed by stream id \
  /// None indicates that the stream was recently added and properties have not been fetched
  pub streams: DashMap<String, Option<Stream>>,
}

enum ClientPartialUpdate {
  Volume(ClientVolume),
  Latency(usize),
  Name(String),
}

enum GroupPartialUpdate {
  Name(String),
  StreamId(String),
  Muted(bool),
}

enum StreamPartialUpdate {
  Properties(StreamProperties),
}

impl State {
  pub(crate) fn handle_result(&self, data: SnapcastResult) {
    match data {
      // client
      SnapcastResult::ClientGetStatus(result) => self.client_upsert(result.client),
      SnapcastResult::ClientSetVolume(id, result) => {
        self.client_partial_update(id, ClientPartialUpdate::Volume(result.volume))
      }
      SnapcastResult::ClientSetLatency(id, result) => {
        self.client_partial_update(id, ClientPartialUpdate::Latency(result.latency))
      }
      SnapcastResult::ClientSetName(id, result) => {
        self.client_partial_update(id, ClientPartialUpdate::Name(result.name))
      }

      // group
      SnapcastResult::GroupGetStatus(result) => {
        let clients = result.group.clients.iter().map(|c| c.id.clone()).collect();
        self.group_upsert(result.group, clients);
      }
      SnapcastResult::GroupSetMute(id, result) => self.group_partial_update(id, GroupPartialUpdate::Muted(result.mute)),
      SnapcastResult::GroupSetStream(id, result) => {
        self.group_partial_update(id, GroupPartialUpdate::StreamId(result.stream_id))
      }
      SnapcastResult::GroupSetName(id, result) => self.group_partial_update(id, GroupPartialUpdate::Name(result.name)),
      SnapcastResult::GroupSetClients(result) => self.full_server_upsert(result.server),

      // server
      SnapcastResult::ServerGetRPCVersion(_) => {}
      SnapcastResult::ServerGetStatus(result) => self.full_server_upsert(result.server),
      SnapcastResult::ServerDeleteClient(result) => self.full_server_upsert(result.server),

      // stream
      SnapcastResult::StreamAddStream(result) => self.stream_upsert(result.id, None),
      SnapcastResult::StreamRemoveStream(result) => {
        self.streams.remove(&result.id);
      }
      SnapcastResult::StreamControl(_) => {}
      SnapcastResult::StreamSetProperty(_) => {}
    };
  }

  pub(crate) fn handle_notification(&self, data: Notification) {
    match data {
      // client
      Notification::ClientOnConnect { params } => self.client_upsert(params.client),
      Notification::ClientOnDisconnect { params } => self.client_remove(params.id),
      Notification::ClientOnVolumeChanged { params } => {
        self.client_partial_update(params.id, ClientPartialUpdate::Volume(params.volume))
      }
      Notification::ClientOnLatencyChanged { params } => {
        self.client_partial_update(params.id, ClientPartialUpdate::Latency(params.latency))
      }
      Notification::ClientOnNameChanged { params } => {
        self.client_partial_update(params.id, ClientPartialUpdate::Name(params.name))
      }

      // group
      Notification::GroupOnMute { params } => {
        self.group_partial_update(params.id, GroupPartialUpdate::Muted(params.mute))
      }
      Notification::GroupOnStreamChanged { params } => {
        self.group_partial_update(params.id, GroupPartialUpdate::StreamId(params.stream_id))
      }
      Notification::GroupOnNameChanged { params } => {
        self.group_partial_update(params.id, GroupPartialUpdate::Name(params.name))
      }

      // server
      Notification::ServerOnUpdate { params } => self.full_server_upsert(params.server),

      // stream
      Notification::StreamOnUpdate { params } => self.stream_upsert(params.stream.id.clone(), Some(params.stream)),
      Notification::StreamOnProperties { params } => {
        self.stream_partial_update(params.id, StreamPartialUpdate::Properties(params.properties))
      }
    };
  }

  fn full_server_upsert(&self, data: Server) {
    self.server_details_upsert(data.server);

    let group_keys: HashSet<&str> = data.groups.iter().map(|g| &*g.id).collect();
    self.groups.retain(|k, _| group_keys.contains(k.as_str()));

    let client_keys: HashSet<&str> = data
      .groups
      .iter()
      .flat_map(|g| g.clients.iter().map(|c| &*c.id))
      .collect();
    self.clients.retain(|k, _| client_keys.contains(k.as_str()));

    for mut group in data.groups {
      let clients: HashSet<String> = group.clients.iter().map(|c| c.id.clone()).collect();

      for client in group.clients.drain(..) {
        self.client_upsert(client);
      }

      self.group_upsert(group, clients);
    }

    let stream_keys: HashSet<&str> = data.streams.iter().map(|s| &*s.id).collect();
    self.streams.retain(|k, _| stream_keys.contains(k.as_str()));

    for stream in data.streams {
      self.stream_upsert(stream.id.clone(), Some(stream));
    }
  }

  // client
  fn client_upsert(&self, client: Client) {
    let entry = self.clients.entry(client.id.clone());
    if let Entry::Occupied(mut entry) = entry {
      let entry = entry.get_mut();
      *entry = client;
    } else {
      entry.insert(client);
    }
  }

  fn client_remove(&self, id: String) {
    self.clients.remove(&id);
  }

  fn client_partial_update(&self, id: String, update: ClientPartialUpdate) {
    let entry = self.clients.entry(id);
    if let Entry::Occupied(mut entry) = entry {
      let entry = entry.get_mut();

      match update {
        ClientPartialUpdate::Volume(volume) => entry.config.volume = volume,
        ClientPartialUpdate::Latency(latency) => entry.config.latency = latency,
        ClientPartialUpdate::Name(name) => entry.config.name = name,
      }
    }
  }

  // group
  fn group_upsert(&self, group: Group, clients: HashSet<String>) {
    let entry = self.groups.entry(group.id.clone());
    if let Entry::Occupied(mut entry) = entry {
      let entry = entry.get_mut();

      entry.name = group.name;
      entry.stream_id = group.stream_id;
      entry.muted = group.muted;
      entry.clients = clients;
    } else {
      entry.insert(StateGroup {
        id: group.id.clone(),
        name: group.name,
        stream_id: group.stream_id.clone(),
        muted: group.muted,
        clients,
      });
    }
  }

  fn group_partial_update(&self, id: String, update: GroupPartialUpdate) {
    let entry = self.groups.entry(id.clone());
    if let Entry::Occupied(mut entry) = entry {
      let entry = entry.get_mut();

      match update {
        GroupPartialUpdate::Name(name) => entry.name = name,
        GroupPartialUpdate::Muted(muted) => entry.muted = muted,
        GroupPartialUpdate::StreamId(stream_id) => {
          entry.stream_id = stream_id;
        }
      }
    }
  }

  // server
  fn server_details_upsert(&self, server: ServerDetails) {
    if self.server.get().is_none() {
      self.server.set(RwLock::new(server)).expect("this should never fail");
    } else {
      let mut entry = self.server.get().unwrap().write().expect("rwlock poisoned");
      *entry = server;
    }
  }

  // stream
  fn stream_upsert(&self, id: String, stream: Option<Stream>) {
    let entry = self.streams.entry(id);
    if let Entry::Occupied(mut entry) = entry {
      let entry = entry.get_mut();
      *entry = stream;
    } else {
      entry.insert(stream);
    }
  }

  fn stream_partial_update(&self, id: String, update: StreamPartialUpdate) {
    let entry = self.streams.entry(id);
    if let Entry::Occupied(mut entry) = entry {
      let entry = entry.get_mut();

      match update {
        StreamPartialUpdate::Properties(properties) => {
          if let Some(entry) = entry {
            entry.properties = Some(properties);
          }
        }
      }
    }
  }
}
