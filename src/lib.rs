#![doc = include_str!("../README.md")]

mod communication;
mod protocol;
mod state;

pub use communication::{ClientError, SnapcastConnection};
pub use protocol::*;
pub use state::{State, StateGroup};
