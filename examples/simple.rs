//! simple example of connecting to a Snapcast server without connection callbacks

use snapcast_control::{SnapcastConnection, ValidMessage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut client = SnapcastConnection::open("127.0.0.1:1705".parse()?)
    .await
    .expect("could not connect to server");

  // client state is updated with each message received
  let state = client.state.clone();

  // state is empty initially, sending the server_get_status request will populate it
  client.server_get_status().await?;

  loop {
    tokio::select! {
      Some(results) = client.recv() => {
        for result in results {
          match result {
            Ok(message) => {
              match message {
                ValidMessage::Result { result, .. } => {
                  println!("received result: {:?}", result);
                }
                ValidMessage::Notification { method, .. } => {
                  println!("received notification: {:?}", method);
                }
              }
            }
            Err(err) => {
              eprintln!("error: {}", err);
            }
          }
        }

        println!("current groups: {:?}", state.groups.iter().map(|g| g.key().clone()).collect::<Vec<_>>());
      }
      _ = tokio::signal::ctrl_c() => {
        println!("shutting down");
        return Ok(());
      }
    }
  }
}
