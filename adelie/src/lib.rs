#![no_std]
#![allow(unused)] // TODO: Remove this later

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;


pub mod memory;
pub mod cartridge;
pub mod instance;
mod util;
