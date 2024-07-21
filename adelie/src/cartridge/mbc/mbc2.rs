use crate::cartridge::{InstantCartridge, MapperType};
use crate::cartridge::mbc::{MBCResult, TYPICAL_ROM_BANK_SIZE, typical_rom_offset, validate};
use crate::instance::io::{CARTRIDGE_RAM_END, CARTRIDGE_RAM_START, CARTRIDGE_ROM_END, CARTRIDGE_ROM_MAIN_BANK_END};
use crate::memory::InstantMemory;

pub struct MBC2<'a> {
    rom: &'a [u8],
    ram: &'a mut [u8; 0x100],
    ram_enabled: bool,
    rom_bank: usize
}

impl MBC2<'_> {
    pub fn new<'a>(rom: &'a [u8], ram: &'a mut [u8; 0x100]) -> MBCResult<MBC2<'a>> {
        let _ = validate(rom, ram, MapperType::MBC2)?;
        Ok(MBC2 { rom, ram, ram_enabled: false, rom_bank: 1 })
    }

    /// Returns the byte and how much to shift the byte
    fn get_sram_byte(&mut self, address: u16) -> (&mut u8, u8) {
        let offset = ((address as usize) >> 1) & (self.ram.len() - 1);
        (&mut self.ram[offset], ((address & 1) << 4) as u8)
    }
}

impl InstantMemory for MBC2<'_> {
    fn read(&mut self, address: u16) -> u8 {
        if address <= CARTRIDGE_ROM_END {
            self.rom[typical_rom_offset(address, self.rom_bank)]
        }
        else if self.ram_enabled {
            let (byte, shift) = self.get_sram_byte(address);
            (*byte >> shift) & 0xF
        }
        else {
            0xFF
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        if address <= CARTRIDGE_ROM_MAIN_BANK_END {
            if (address & 0x100) == 0 {
                self.ram_enabled = data == 0xA && !self.ram.is_empty();
            }
            else {
                self.rom_bank = ((data as usize) & 0xF).clamp(1, self.rom.len() / TYPICAL_ROM_BANK_SIZE);
            }
        }
        else if self.ram_enabled && (CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END).contains(&address) {
            let (byte, shift) = self.get_sram_byte(address);
            *byte = (*byte & !(0xF << shift)) | (data & 0xF) << shift;
        }
    }
}

impl InstantCartridge for MBC2<'_> {
    fn rom_bank_size(&self) -> Option<usize> {
        Some(TYPICAL_ROM_BANK_SIZE)
    }
    fn rom_data(&self) -> Option<&[u8]> {
        Some(self.rom)
    }
    fn ram_data(&self) -> Option<&[u8]> {
        Some(self.ram)
    }
    fn ram_data_mut(&mut self) -> Option<&mut [u8]> {
        Some(self.ram)
    }
}
