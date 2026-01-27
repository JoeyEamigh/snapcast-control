//! example of connecting to a Snapcast server with connection status callbacks

use snapcast_control::{ConnectionStatus, SnapcastConnection, ValidMessage};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // create a channel to receive connection status events
  let (status_tx, mut status_rx) = mpsc::unbounded_channel::<ConnectionStatus>();

  let mut client = SnapcastConnection::builder()
    .on_status_change(move |status| {
      // send status to the channel (ignore errors if receiver is dropped)
      let _ = status_tx.send(status);
    })
    .connect("127.0.0.1:1705".parse()?)
    .await?;

  // request initial state
  client.server_get_status().await?;

  loop {
    tokio::select! {
      // handle connection status changes
      Some(status) = status_rx.recv() => {
        match status {
          ConnectionStatus::Connected => {
            println!("[connection] connected to server");
            // refresh state after reconnection
            if let Err(e) = client.server_get_status().await {
              eprintln!("[connection] failed to refresh state: {}", e);
            }
          }
          ConnectionStatus::Disconnected => {
            println!("[connection] disconnected from server, reconnecting...");
          }
          ConnectionStatus::ReconnectFailed => {
            println!("[connection] reconnection attempt failed, will retry...");
          }
        }
      }

      // handle messages from the server
      Some(results) = client.recv() => {
        for result in results {
          match result {
            Ok(message) => {
              match message {
                ValidMessage::Result { result, .. } => {
                  println!("[message] result: {:?}", result);
                }
                ValidMessage::Notification { method, .. } => {
                  println!("[message] notification: {:?}", method);
                }
              }
            }
            Err(err) => {
              eprintln!("[error] {}", err);
            }
          }
        }
      }

      // handle ctrl+c
      _ = tokio::signal::ctrl_c() => {
        println!("shutting down");
        return Ok(());
      }
    }
  }
}
