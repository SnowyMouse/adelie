#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod memory;
pub mod cartridge;
pub mod instance;
mod util;
