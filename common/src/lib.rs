//! `core` houses the common traits and types between the CPU usage sender and the CPU usage receiver.

#![no_std]

use heapless::Vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CPUUsage {
    pub id: u8,
    pub usage: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Packet {
    pub cores: Vec<CPUUsage, 32>
}
pub const RES_WIDTH: u16 = 64;
pub const RES_HEIGHT: u16 = 32;
pub const TOTAL_LEDS: usize = const { RES_WIDTH as usize * RES_HEIGHT as usize };
pub const CPU_WIDTH: u8 = 10;
pub const CPU_HEIGHT: u8 = 16;
pub const THREAD_COUNT: usize = 24;
// If we assume 10x16 LEDs represent a single CPU core. We have 12 cores so we need to get the LED slice for each one.
// Given our 64*32 LED matrix we leave the last four rows for 60*32 LEDs used.
/// Generate the LED indexes of the specific CPU
pub const fn generate_indexes(cpu_index: usize) -> [usize; (CPU_WIDTH * CPU_HEIGHT) as usize] {
    let mut indexes = [0; (CPU_WIDTH * CPU_HEIGHT) as usize];
    let mut idx = if cpu_index < 6 { 0 } else { TOTAL_LEDS / 2 };
    let rel_cpu = cpu_index % 6;
    let mut len = 0;
    loop {
        let min_x = (CPU_WIDTH as usize) * rel_cpu;
        let max_x = min_x + CPU_WIDTH as usize;
        let rel_idx = idx % (RES_WIDTH as usize);
        if rel_idx >= min_x && rel_idx < max_x {
            indexes[len] = idx;
            len += 1;
            if len as u8 == CPU_WIDTH * CPU_HEIGHT {
                break;
            }
        }
        idx += 1;
    }
    indexes
}