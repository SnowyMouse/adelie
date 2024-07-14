use crate::memory::Memory;

/// Denotes a cartridge, which contains the game.
pub trait Cartridge: Memory {
    /// Return true if the cart reset line is set.
    ///
    /// Most cartridges don't implement this.
    fn reset(&self) -> bool {
        false
    }

    /// Get the size of a bank in ROM.
    ///
    /// Note that physical cartridges may return None.
    fn rom_bank_size(&self) -> Option<usize> {
        None
    }

    /// Get a reference to the ROM data.
    ///
    /// Note that physical cartridges may return None.
    fn rom_data(&self) -> Option<&[u8]> {
        None
    }

    /// Get a mutable reference to the ROM data.
    ///
    /// Note that physical cartridges may return None.
    fn rom_data_mut(&mut self) -> Option<&mut [u8]> {
        None
    }

    /// Get the size of a bank in RAM.
    ///
    /// Note that physical cartridges as well as cartridges with no RAM may return None.
    fn ram_bank_size(&self) -> Option<usize> {
        None
    }

    /// Get a reference to the RAM data.
    ///
    /// Note that physical cartridges as well as cartridges with no RAM may return None.
    fn ram_data(&self) -> Option<&[u8]> {
        None
    }

    /// Get a mutable reference to the RAM data.
    ///
    /// Note that physical cartridges as well as cartridges with no RAM may return None.
    fn ram_data_mut(&mut self) -> Option<&mut [u8]> {
        None
    }
}

/// Denotes the state a cartridge is not present.
#[derive(Copy, Clone)]
pub struct NullCartridge;
impl Cartridge for NullCartridge {}
impl Memory for NullCartridge {
    fn read(&mut self, _address: u16) -> u8 {
        0xFF
    }
    fn write(&mut self, _address: u16, _data: u8) {}
}

#[cfg(feature = "alloc")]
impl Cartridge for alloc::boxed::Box<dyn Cartridge> {
    fn reset(&self) -> bool {
        self.as_ref().reset()
    }
    fn rom_bank_size(&self) -> Option<usize> {
        self.as_ref().rom_bank_size()
    }
    fn rom_data(&self) -> Option<&[u8]> {
        self.as_ref().rom_data()
    }
    fn rom_data_mut(&mut self) -> Option<&mut [u8]> {
        self.as_mut().rom_data_mut()
    }
    fn ram_bank_size(&self) -> Option<usize> {
        self.as_ref().ram_bank_size()
    }
    fn ram_data(&self) -> Option<&[u8]> {
        self.as_ref().ram_data()
    }
    fn ram_data_mut(&mut self) -> Option<&mut [u8]> {
        self.as_mut().ram_data_mut()
    }
}

#[cfg(feature = "alloc")]
impl Memory for alloc::boxed::Box<dyn Cartridge> {
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
