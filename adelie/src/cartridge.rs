pub mod mbc;

use core::fmt::{Display, Formatter};
use crate::memory::{InstantMemory, Memory};

/// Denotes a cartridge that can be used with the emulator.
pub trait Cartridge: Memory {
    /// Return true if the cart reset line is set.
    fn reset_line_set(&self) -> bool {
        false
    }
}

/// Denotes an emulated cartridge.
pub trait DebugCartridge: InstantMemory {
    /// Return true if the cart reset line is set.
    fn reset_line_set(&self) -> bool {
        false
    }

    /// Get the size of a bank in ROM.
    ///
    /// Returns `None` if no ROM is present or ROM is not bankable.
    fn rom_bank_size(&self) -> Option<usize>;

    /// Get the current ROM bank.
    ///
    /// Returns `None` if no ROM is present or ROM is not bankable.
    fn rom_bank(&self) -> Option<usize>;

    /// Get a reference to the ROM data.
    ///
    /// Returns `None` if no ROM is present.
    fn rom_data(&self) -> Option<&[u8]>;

    /// Get the size of a bank in RAM.
    ///
    /// Returns `None` if no RAM is present or RAM is not bankable.
    fn ram_bank_size(&self) -> Option<usize>;

    /// Get the current RAM bank.
    ///
    /// Returns `None` if no RAM is present or RAM is not bankable.
    fn ram_bank(&self) -> Option<usize>;

    /// Get a reference to the RAM data.
    ///
    /// Returns `None` if no RAM is present.
    fn ram_data(&self) -> Option<&[u8]>;

    /// Get a mutable reference to the RAM data.
    ///
    /// Returns `None` if no RAM is present.
    fn ram_data_mut(&mut self) -> Option<&mut [u8]>;
}

/// Denotes the state a cartridge is not present.
#[derive(Copy, Clone)]
pub struct NullCartridge;
impl DebugCartridge for NullCartridge {
    fn rom_bank_size(&self) -> Option<usize> {
        None
    }

    fn rom_bank(&self) -> Option<usize> {
        None
    }

    fn rom_data(&self) -> Option<&[u8]> {
        None
    }

    fn ram_bank_size(&self) -> Option<usize> {
        None
    }

    fn ram_bank(&self) -> Option<usize> {
        None
    }

    fn ram_data(&self) -> Option<&[u8]> {
        None
    }

    fn ram_data_mut(&mut self) -> Option<&mut [u8]> {
        None
    }
}
impl InstantMemory for NullCartridge {
    fn read(&mut self, _address: u16) -> u8 {
        0xFF
    }
    fn write(&mut self, _address: u16, _data: u8) {}
}

#[cfg(feature = "alloc")]
impl DebugCartridge for alloc::boxed::Box<dyn DebugCartridge> {
    fn rom_bank_size(&self) -> Option<usize> {
        self.as_ref().rom_bank_size()
    }

    fn rom_bank(&self) -> Option<usize> {
        self.as_ref().rom_bank()
    }

    fn rom_data(&self) -> Option<&[u8]> {
        self.as_ref().rom_data()
    }

    fn ram_bank_size(&self) -> Option<usize> {
        self.as_ref().ram_bank_size()
    }

    fn ram_bank(&self) -> Option<usize> {
        self.as_ref().ram_bank()
    }

    fn ram_data(&self) -> Option<&[u8]> {
        self.as_ref().ram_data()
    }

    fn ram_data_mut(&mut self) -> Option<&mut [u8]> {
        self.as_mut().ram_data_mut()
    }
}

#[cfg(feature = "alloc")]
impl InstantMemory for alloc::boxed::Box<dyn DebugCartridge> {
    fn read(&mut self, address: u16) -> u8 {
        self.as_mut().read(address)
    }

    fn write(&mut self, address: u16, data: u8) {
        self.as_mut().write(address, data)
    }

    fn get_bank(&self) -> Option<usize> {
        self.as_ref().get_bank()
    }

    fn get_memory(&self) -> Option<&[u8]> {
        self.as_ref().get_memory()
    }

    fn get_memory_mut(&mut self) -> Option<&mut [u8]> {
        self.as_mut().get_memory_mut()
    }
}

/// Cartridge info from a ROM.
#[must_use]
pub struct CartridgeHeaderInfo {
    /// Mapper type to use.
    pub mapper_type: MapperType,

    /// Size of data needed for ROM.
    pub rom_size: usize,

    /// Size of data needed for writable memory.
    pub ram_size: usize,

    /// The cartridge has a built-in real-time clock.
    pub has_rtc: bool,

    /// The cartridge's writable memory is saved when the game is powered off.
    pub has_save_data: bool,

    /// The cartridge has rumble.
    pub has_rumble: bool,

    /// Cartridge will boot in a retail console.
    pub bootable: bool
}
impl CartridgeHeaderInfo {
    pub fn read_cartridge_header(header: &[u8; 0x50]) -> Result<Self, CartridgeHeaderError> {
        #[derive(Default)]
        struct CartridgeType {
            mapper: MapperType,
            has_ram: bool,
            has_save_data: bool,
            has_rumble: bool,
            has_rtc: bool
        }

        let cartridge_type = match header[0x47] {
            // ROM ONLY
            0x00 => CartridgeType { mapper: MapperType::ROMOnly, ..CartridgeType::default() },

            // MBC1
            0x01 => CartridgeType { mapper: MapperType::MBC1, ..CartridgeType::default() },

            // MBC1+RAM
            0x02 => CartridgeType { mapper: MapperType::MBC1, has_ram: true, ..CartridgeType::default() },

            // MBC1+RAM+BATTERY
            0x03 => CartridgeType { mapper: MapperType::MBC1, has_ram: true, has_save_data: true, ..CartridgeType::default() },

            // MBC2
            0x05 => CartridgeType { mapper: MapperType::MBC2, ..CartridgeType::default() },

            // MBC2+BATTERY
            0x06 => CartridgeType { mapper: MapperType::MBC2, has_save_data: true, ..CartridgeType::default() },

            // ROM+RAM
            0x08 => CartridgeType { mapper: MapperType::ROMOnly, has_ram: true, ..CartridgeType::default() },

            // ROM+RAM+BATTERY
            0x09 => CartridgeType { mapper: MapperType::ROMOnly, has_ram: true, has_save_data: true, ..CartridgeType::default() },

            // MBC3+TIMER+BATTERY
            0x0F => CartridgeType { mapper: MapperType::MBC3, has_rtc: true, ..CartridgeType::default() },

            // MBC3+TIMER+RAM+BATTERY
            0x10 => CartridgeType { mapper: MapperType::MBC3, has_ram: true, has_save_data: true, has_rtc: true, ..CartridgeType::default() },

            // MBC3
            0x11 => CartridgeType { mapper: MapperType::MBC3, ..CartridgeType::default() },

            // MBC3+RAM
            0x12 => CartridgeType { mapper: MapperType::MBC3, has_ram: true, ..CartridgeType::default() },

            // MBC3+RAM+BATTERY
            0x13 => CartridgeType { mapper: MapperType::MBC3, has_ram: true, has_save_data: true, ..CartridgeType::default() },

            // MBC5
            0x19 => CartridgeType { mapper: MapperType::MBC5, ..CartridgeType::default() },

            // MBC5+RAM
            0x1A => CartridgeType { mapper: MapperType::MBC5, has_ram: true, ..CartridgeType::default() },

            // MBC5+RAM+BATTERY
            0x1B => CartridgeType { mapper: MapperType::MBC5, has_ram: true, has_save_data: true, ..CartridgeType::default() },

            // MBC5+RUMBLE
            0x1C => CartridgeType { mapper: MapperType::MBC5, has_rumble: true, ..CartridgeType::default() },

            // MBC5+RUMBLE+RAM
            0x1D => CartridgeType { mapper: MapperType::MBC5, has_ram: true, has_rumble: true, ..CartridgeType::default() },

            // MBC5+RUMBLE+RAM+BATTERY
            0x1E => CartridgeType { mapper: MapperType::MBC5, has_ram: true, has_save_data: true, has_rumble: true, ..CartridgeType::default() },

            // MBC6
            0x20 => CartridgeType { mapper: MapperType::MBC6, ..CartridgeType::default() },

            // MBC7+SENSOR+RUMBLE+RAM+BATTERY
            0x22 => CartridgeType { mapper: MapperType::MBC7, has_ram: true, has_save_data: true, has_rumble: true, ..CartridgeType::default() },

            n => return Err(CartridgeHeaderError::UnknownMapper(n))
        };

        let writable_memory_size = if cartridge_type.mapper == MapperType::MBC2 {
            256 // 512 half bytes
        }
        else if cartridge_type.mapper == MapperType::MBC7 {
            256 // 256 bytes
        }
        else if cartridge_type.has_ram {
            match header[0x49] {
                2 => 8 * 1024,
                3 => 32 * 1024,
                4 => 128 * 1024,
                5 => 64 * 1024,
                n => return Err(CartridgeHeaderError::UnknownRAMSize(n))
            }
        }
        else {
            0
        };

        let rom_size_val = header[0x48];
        let rom_size = match rom_size_val {
            0..=8 => 32 * 1024 * (rom_size_val as usize),
            n => return Err(CartridgeHeaderError::UnknownROMSize(n))
        };

        let valid_logo = header[0x4..=0x33] == [
            0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
            0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
            0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E
        ];

        let mut checksum = 0u8;
        for i in &header[0x34..=0x4C] {
            checksum = checksum.wrapping_sub(*i).wrapping_sub(1);
        }

        let checksum_matches = valid_logo && checksum == header[0x4D];

        Ok(Self {
            mapper_type: cartridge_type.mapper,
            rom_size,
            ram_size: writable_memory_size,
            has_rumble: cartridge_type.has_rumble,
            has_save_data: cartridge_type.has_save_data,
            has_rtc: cartridge_type.has_rtc,
            bootable: valid_logo && checksum_matches,
        })
    }
}

#[derive(Default, PartialEq, Debug)]
pub enum MapperType {
    #[default]
    ROMOnly,
    MBC1,
    MBC2,
    MBC3,
    MBC5,
    MBC6,
    MBC7
}

#[derive(Debug, PartialEq)]
pub enum CartridgeHeaderError {
    UnknownMapper(u8),
    UnknownRAMSize(u8),
    UnknownROMSize(u8),
}
impl Display for CartridgeHeaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownMapper(m) => f.write_fmt(format_args!("Unknown mapper ({m:#02X})")),
            Self::UnknownRAMSize(m) => f.write_fmt(format_args!("Unknown RAM size ({m:#02X})")),
            Self::UnknownROMSize(m) => f.write_fmt(format_args!("Unknown ROM size ({m:#02X})")),
        }
    }
}

/// Used for wrapping a DebugCartridge with an address bus, thus implementing [`Memory`].
#[derive(Copy, Clone)]
pub struct EmulatedCartridge<Cart: DebugCartridge> {
    cartridge: Cart,
    address: u16
}
impl<Cart: DebugCartridge> EmulatedCartridge<Cart> {
    pub fn new(cartridge: Cart) -> EmulatedCartridge<Cart> {
        EmulatedCartridge { cartridge, address: 0 }
    }
}
impl<Cart: DebugCartridge> DebugCartridge for EmulatedCartridge<Cart> {
    fn rom_bank_size(&self) -> Option<usize> {
        self.cartridge.rom_bank_size()
    }

    fn rom_bank(&self) -> Option<usize> {
        self.cartridge.rom_bank()
    }

    fn rom_data(&self) -> Option<&[u8]> {
        self.cartridge.rom_data()
    }

    fn ram_bank_size(&self) -> Option<usize> {
        self.cartridge.ram_bank_size()
    }

    fn ram_bank(&self) -> Option<usize> {
        self.cartridge.ram_bank()
    }

    fn ram_data(&self) -> Option<&[u8]> {
        self.cartridge.ram_data()
    }

    fn ram_data_mut(&mut self) -> Option<&mut [u8]> {
        self.cartridge.ram_data_mut()
    }
}

impl<C: DebugCartridge> InstantMemory for EmulatedCartridge<C> {
    fn read(&mut self, address: u16) -> u8 {
        self.cartridge.read(address)
    }

    fn write(&mut self, address: u16, data: u8) {
        self.cartridge.write(address, data)
    }

    fn get_bank(&self) -> Option<usize> {
        self.cartridge.get_bank()
    }

    fn get_memory(&self) -> Option<&[u8]> {
        self.cartridge.get_memory()
    }

    fn get_memory_mut(&mut self) -> Option<&mut [u8]> {
        self.cartridge.get_memory_mut()
    }
}
impl<C: DebugCartridge> Memory for EmulatedCartridge<C> {
    fn set_data_lines(&mut self, address: u16, write: bool, data_in: u8) {
        self.address = address;
        if write {
            self.cartridge.write(address, data_in)
        }
    }

    fn read_out(&mut self) -> u8 {
        self.cartridge.read(self.address)
    }
}
impl<C: DebugCartridge> Cartridge for EmulatedCartridge<C> {
    fn reset_line_set(&self) -> bool {
        self.cartridge.reset_line_set()
    }
}
