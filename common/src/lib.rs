//! `core` houses the common traits and types between the CPU usage sender and the CPU usage receiver.

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CPUUsage {
    pub id: u8,
    pub usage: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Packet {
    pub cores: Vec<CPUUsage>,
}
