use crate::cartridge::{DebugCartridge, MapperType};
use crate::cartridge::mbc::{MBCResult, TYPICAL_RAM_BANK_SIZE, typical_ram_offset, TYPICAL_ROM_BANK_SIZE, typical_rom_offset, validate};
use crate::instance::io::CARTRIDGE_ROM_END;
use crate::memory::InstantMemory;

pub struct MBC5<'a> {
    rom: &'a [u8],
    ram: &'a mut [u8],
    ram_enabled: bool,
    rom_bank: usize,
    ram_bank: usize,
    rumble: Option<bool>,
    rom_bank_count: usize,
    ram_bank_count: usize
}

impl MBC5<'_> {
    pub fn new<'a>(rom: &'a [u8], ram: &'a mut [u8]) -> MBCResult<MBC5<'a>> {
        let info = validate(rom, ram, MapperType::MBC5)?;

        Ok(
            MBC5 {
            ram_enabled: false,
            rom_bank: 1,
            rom_bank_count: rom.len() / TYPICAL_ROM_BANK_SIZE,
            ram_bank_count: ram.len() / TYPICAL_RAM_BANK_SIZE,
            ram_bank: 0,
            rom,
            ram,
            rumble: if info.has_rumble { Some(false) } else { None } }
        )
    }
    pub fn rumble_on(&self) -> Option<bool> {
        self.rumble
    }
}

impl InstantMemory for MBC5<'_> {
    fn read(&mut self, address: u16) -> u8 {
        if address <= CARTRIDGE_ROM_END {
            self.rom[typical_rom_offset(address, self.rom_bank)]
        }
        else if self.ram_enabled {
            self.ram[typical_ram_offset(address, self.ram_bank)]
        }
        else {
            0xFF
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        if address <= 0x1FFF {
            let data = data & 0xF;
            if data == 0xA && !self.ram.is_empty() {
                self.ram_enabled = true;
            }
            else if data == 0x0 {
                self.ram_enabled = false;
            }
        }
        else if address <= 0x2FFF {
            self.rom_bank &= 0x100;
            self.rom_bank |= (data as usize) & (self.rom_bank_count - 1);
        }
        else if address <= 0x3FFF {
            self.rom_bank &= 0x0FF;
            self.rom_bank |= ((data as usize) << 8) & (self.rom_bank_count - 1);
        }
        else if address <= 0x5FFF {
            // NOTE: If rumble is present, the third bit is unavailable for RAM and is instead
            // diverted into controlling the motor.
            const RUMBLE_BIT: u8 = 1 << 3;
            if !self.ram.is_empty() {
                let data = if self.rumble.is_some() {
                    data & !RUMBLE_BIT
                }
                else {
                    data
                };
                self.ram_bank = (data as usize) & (self.ram_bank_count - 1);
            }
            if let Some(n) = self.rumble.as_mut() {
                *n = (data & RUMBLE_BIT) != 0;
            }
        }
        else if address <= 0x7FFF {
            // do nothing
        }
        else if self.ram_enabled {
            self.ram[typical_ram_offset(address, self.ram_bank)] = data
        }
    }
}

impl DebugCartridge for MBC5<'_> {
    fn rom_bank_size(&self) -> Option<usize> {
        Some(TYPICAL_ROM_BANK_SIZE)
    }

    fn rom_bank(&self) -> Option<usize> {
        Some(self.rom_bank)
    }

    fn rom_data(&self) -> Option<&[u8]> {
        Some(self.rom)
    }
    fn ram_bank_size(&self) -> Option<usize> {
        Some(TYPICAL_RAM_BANK_SIZE)
    }

    fn ram_bank(&self) -> Option<usize> {
        if self.ram.is_empty() {
            None
        }
        else {
            Some(self.ram_bank)
        }
    }

    fn ram_data(&self) -> Option<&[u8]> {
        return_ram_if_present!(self)
    }
    fn ram_data_mut(&mut self) -> Option<&mut [u8]> {
        return_ram_if_present!(self)
    }
}
