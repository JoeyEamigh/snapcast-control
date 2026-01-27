# snapcast-control

`snapcast-control` is a Rust api client for [Snapcast](https://github.com/badaix/snapcast). It supports all features of the Snapcast JSON-RPC API as of version 0.28.0 (2024/6/25).

Documentation is available at [docs.rs](https://docs.rs/snapcast-control).

## Features

- [x] native rust types for all api requests and responses
- [x] tokio-based async client
- [x] client with helper methods for all api requests
- [x] automatic socket reconnection via [stubborn-io](https://github.com/craftytrickster/stubborn-io)
- [x] connection status callbacks for monitoring connect/disconnect events

## Installation

```sh
cargo add snapcast-control
```

## Usage

The best example of this crate's usage is [snapcast-multiroom](https://github.com/JoeyEamigh/snapcast-multiroom), the project I designed it for. There are also basic examples in the `examples/` dir.

### Basic Example

```rust
use snapcast_control::{SnapcastConnection, ValidMessage};

#[tokio::main]
async fn main() {
  let mut client = SnapcastConnection::open("127.0.0.1:1705".parse().expect("could not parse socket address")).await.expect("could not connect to server");

  // client state is updated with each message received
  let state = client.state.clone();

  // state is empty initially, sending the server_get_status request will populate it
  client.server_get_status().await.expect("could not send request");

  loop {
    tokio::select! {
      // recv returns a batch of results (Vec<Result<ValidMessage, ClientError>>)
      Some(results) = client.recv() => {
        for result in results {
          match result {
            Ok(message) => {
              // handle each message
              match message {
                ValidMessage::Result { id, jsonrpc, result } => {},
                ValidMessage::Notification { method, jsonrpc } => {},
              }
            }
            Err(err) => {
              // handle error
            }
          }
        }
      },
      _ = tokio::signal::ctrl_c() => {
        break;
      }
    }
  }
}
```

### Connection Status Callbacks

You can monitor connection status changes using the builder pattern:

```rust
use snapcast_control::{SnapcastConnection, ConnectionStatus};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut client = SnapcastConnection::builder()
    .on_status_change(|status| {
      match status {
        ConnectionStatus::Connected => {
          println!("connected to Snapcast server");
        }
        ConnectionStatus::Disconnected => {
          println!("disconnected from server, reconnecting...");
        }
        ConnectionStatus::ReconnectFailed => {
          println!("reconnection attempt failed, will retry...");
        }
      }
    })
    .connect("127.0.0.1:1705".parse()?)
    .await?;

  // use the client as normal
  client.server_get_status().await?;

  // ...

  Ok(())
}
```

You can also set individual callbacks for each event:

```rust
use snapcast_control::SnapcastConnection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let client = SnapcastConnection::builder()
    .on_connect(|| println!("Connected!"))
    .on_disconnect(|| println!("Disconnected!"))
    .on_reconnect_failed(|| println!("Reconnect attempt failed!"))
    .connect("127.0.0.1:1705".parse()?)
    .await?;

  Ok(())
}
```

Note: Callbacks are invoked synchronously from the stubborn-io reconnection logic, so they should be fast and non-blocking, ideally just a notifier.
