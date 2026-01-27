#![doc = include_str!("../README.md")]

mod communication;
mod protocol;
mod state;

pub use communication::{ClientError, ConnectionCallback, SnapcastConnection};
pub use protocol::*;
pub use state::{State, StateGroup};
