use crate::cartridge::{DebugCartridge, MapperType};
use crate::cartridge::mbc::{MBCResult, typical_ram_offset, validate};
use crate::instance::io::{CARTRIDGE_ROM_MAIN_BANK_END, CARTRIDGE_ROM_END};
use crate::memory::InstantMemory;

pub struct NoROM<'a> {
    rom: &'a [u8],
    ram: &'a mut [u8]
}

impl NoROM<'_> {
    pub fn new<'a>(rom: &'a [u8], ram: &'a mut [u8]) -> MBCResult<NoROM<'a>> {
        let _ = validate(rom, ram, MapperType::ROMOnly)?;
        Ok(NoROM { rom, ram })
    }
}

impl InstantMemory for NoROM<'_> {
    fn read(&mut self, address: u16) -> u8 {
        if address <= CARTRIDGE_ROM_END {
            self.rom[address as usize]
        }
        else if self.ram.is_empty() {
            0xFF
        }
        else {
            self.ram[typical_ram_offset(address, 0)]
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        if address <= CARTRIDGE_ROM_MAIN_BANK_END || self.ram.is_empty() {
            // do nothing
        }
        else {
            self.ram[typical_ram_offset(address, 0)] = data;
        }
    }
}

impl DebugCartridge for NoROM<'_> {
    fn rom_bank_size(&self) -> Option<usize> {
        None
    }

    fn rom_bank(&self) -> Option<usize> {
        None
    }

    fn rom_data(&self) -> Option<&[u8]> {
        Some(self.rom)
    }

    fn ram_bank_size(&self) -> Option<usize> {
        None
    }

    fn ram_bank(&self) -> Option<usize> {
        None
    }

    fn ram_data(&self) -> Option<&[u8]> {
        return_ram_if_present!(self)
    }
    fn ram_data_mut(&mut self) -> Option<&mut [u8]> {
        return_ram_if_present!(self)
    }
}
