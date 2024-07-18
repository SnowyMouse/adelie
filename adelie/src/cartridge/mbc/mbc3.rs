use crate::cartridge::{Cartridge, MapperType};
use crate::cartridge::mbc::{CartridgeLoadError, MBCResult, TYPICAL_RAM_BANK_SIZE, typical_ram_offset, TYPICAL_ROM_BANK_SIZE, typical_rom_offset, validate};
use crate::instance::io::CARTRIDGE_ROM_END;
use crate::memory::Memory;

pub struct MBC3<'a> {
    rom: &'a [u8],
    ram: &'a mut [u8],
    ram_enabled: bool,
    rom_bank: usize,
    ram_mode: MBC3RAMMode,
    latched: bool,

    rtc: Option<RTCData>,
    latched_rtc: Option<RTCData>
}

#[derive(Copy, Clone, Default)]
pub struct RTCData {
    pub seconds: u8,
    pub minutes: u8,
    pub hours: u8,
    pub days_low: u8,
    pub flags: u8
}

impl MBC3<'_> {
    pub fn new<'a>(rom: &'a [u8], ram: &'a mut [u8], rtc: Option<RTCData>) -> MBCResult<MBC3<'a>> {
        let info = validate(rom, ram, MapperType::MBC3)?;

        let (rtc, rtc_latched) = if info.has_rtc {
            (Some(rtc.unwrap_or_default()), Some(RTCData::default()))
        }
        else {
            (None, None)
        };

        if !ram.is_empty() && (ram.len() / TYPICAL_RAM_BANK_SIZE) != 4 {
            return Err(CartridgeLoadError::IncorrectRAMSize { expected: TYPICAL_RAM_BANK_SIZE * 4, actual: ram.len() })
        }

        Ok(MBC3 { rom, ram, ram_enabled: false, rom_bank: 1, latched: true, ram_mode: MBC3RAMMode::RAMBank(1), rtc, latched_rtc: rtc_latched })
    }
    pub fn get_rtc(&self) -> Option<RTCData> {
        self.rtc
    }
    pub fn get_latched_rtc(&self) -> Option<RTCData> {
        self.latched_rtc
    }
    pub fn set_rtc(&mut self, data: RTCData) {
        self.rtc = Some(data);
    }
    pub fn set_latched_rtc(&mut self, data: RTCData) {
        self.latched_rtc = Some(data);
    }
}

impl Memory for MBC3<'_> {
    fn read(&mut self, address: u16) -> u8 {
        if address <= CARTRIDGE_ROM_END {
            self.rom[typical_rom_offset(address, self.rom_bank)]
        }
        else if self.ram_enabled {
            match self.ram_mode {
                MBC3RAMMode::RAMBank(n) => if self.ram.is_empty() { 0xFF } else { self.ram[typical_ram_offset(address, n)] },
                MBC3RAMMode::RTCSeconds => self.latched_rtc.unwrap().seconds,
                MBC3RAMMode::RTCMinutes => self.latched_rtc.unwrap().minutes,
                MBC3RAMMode::RTCHours => self.latched_rtc.unwrap().hours,
                MBC3RAMMode::RTCDaysLow => self.latched_rtc.unwrap().days_low,
                MBC3RAMMode::RTCFlags => self.latched_rtc.unwrap().flags,
            }
        }
        else {
            0xFF
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        if address <= 0x1FFF {
            if data == 0xA && (!self.ram.is_empty() || self.rtc.is_none()) {
                // has to be enabled for either RAM or RTC to work, so even if there is no RAM but there's RTC (and vice versa), we need this
                self.ram_enabled = true;
            }
            else if data == 0x0 {
                self.ram_enabled = false;
            }
        }
        else if address <= 0x3FFF {
            self.rom_bank = (data & 0x7F).clamp(1, (self.rom.len() / TYPICAL_ROM_BANK_SIZE) as u8) as usize;
        }
        else if address <= 0x5FFF {
            if self.rtc.is_none() {
                // TODO: Determine if this register is still modified even on non-RTC MBC3s
                if data < 4 {
                    self.ram_mode = MBC3RAMMode::RAMBank(data as usize);
                }
            }
            else {
                self.ram_mode = match data {
                    0x00..=0x03 => MBC3RAMMode::RAMBank(data as usize),
                    0x08 => MBC3RAMMode::RTCSeconds,
                    0x09 => MBC3RAMMode::RTCMinutes,
                    0x0A => MBC3RAMMode::RTCHours,
                    0x0B => MBC3RAMMode::RTCDaysLow,
                    0x0C => MBC3RAMMode::RTCFlags,
                    _ => return
                }
            }
        }
        else if address <= 0x7FFF {
            if data == 0x00 {
                self.latched = false;
            }
            if data == 0x01 && !self.latched {
                self.latched = true;
                self.latched_rtc = self.rtc;
            }
        }
        else if self.ram_enabled {
            let modifiable_rtc = self.rtc.as_mut();
            match self.ram_mode {
                MBC3RAMMode::RAMBank(b) => if !self.ram.is_empty() { self.ram[typical_ram_offset(address, b)] = data },
                MBC3RAMMode::RTCSeconds => modifiable_rtc.unwrap().seconds = data,
                MBC3RAMMode::RTCMinutes => modifiable_rtc.unwrap().minutes = data,
                MBC3RAMMode::RTCHours => modifiable_rtc.unwrap().hours = data,
                MBC3RAMMode::RTCDaysLow => modifiable_rtc.unwrap().days_low = data,
                MBC3RAMMode::RTCFlags => modifiable_rtc.unwrap().flags = data,
            }
        }
    }
}

#[derive(Copy, Clone)]
enum MBC3RAMMode {
    RAMBank(usize),
    RTCSeconds,
    RTCMinutes,
    RTCHours,
    RTCDaysLow,
    RTCFlags
}



impl Cartridge for MBC3<'_> {
    fn rom_bank_size(&self) -> Option<usize> {
        Some(TYPICAL_ROM_BANK_SIZE)
    }
    fn rom_data(&self) -> Option<&[u8]> {
        Some(self.rom)
    }
    fn ram_bank_size(&self) -> Option<usize> {
        Some(TYPICAL_RAM_BANK_SIZE)
    }
    fn ram_data(&self) -> Option<&[u8]> {
        Some(self.ram)
    }
    fn ram_data_mut(&mut self) -> Option<&mut [u8]> {
        Some(self.ram)
    }
}
