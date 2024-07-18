pub mod no_rom;
pub mod mbc2;
pub mod mbc3;
pub mod mbc5;

use core::fmt::{Display, Formatter};
use crate::cartridge::{CartridgeHeaderInfo, MapperType};
use crate::instance::io::CARTRIDGE_ROM_MAIN_BANK_END;

pub type MBCResult<T> = Result<T, CartridgeLoadError>;

const TYPICAL_ROM_BANK_SIZE: usize = 0x4000;
const TYPICAL_ROM_ADDRESS_MASK: usize = TYPICAL_ROM_BANK_SIZE - 1;
const TYPICAL_RAM_BANK_SIZE: usize = 0x2000;
const TYPICAL_RAM_ADDRESS_MASK: usize = TYPICAL_RAM_BANK_SIZE - 1;


fn get_header_data_from_rom(rom: &[u8]) -> Result<CartridgeHeaderInfo, CartridgeLoadError> {
    let Some(header_data) = rom.get(0x100..0x150) else {
        return Err(CartridgeLoadError::CannotIdentifyCartridgeType)
    };
    CartridgeHeaderInfo::read_cartridge_header(header_data.try_into().unwrap())
        .map_err(|_| CartridgeLoadError::CannotIdentifyCartridgeType)
}

fn validate(rom: &[u8], ram: &[u8], expected_mapper_type: MapperType) -> Result<CartridgeHeaderInfo, CartridgeLoadError> {
    let info = get_header_data_from_rom(rom)?;

    if info.mapper_type != expected_mapper_type {
        return Err(CartridgeLoadError::IncorrectMapper { expected: expected_mapper_type, actual: info.mapper_type })
    }

    if rom.len() != info.rom_size {
        return Err(CartridgeLoadError::IncorrectROMSize { expected: info.rom_size, actual: rom.len() })
    }

    if ram.len() != info.ram_size {
        return Err(CartridgeLoadError::IncorrectRAMSize { expected: info.ram_size, actual: ram.len() })
    }

    Ok(info)
}

#[inline(always)]
const fn typical_rom_offset(address: u16, bank: usize) -> usize {
    double_bank_rom_offset(address, 0, bank)
}

#[inline(always)]
const fn double_bank_rom_offset(address: u16, bank0: usize, bank1: usize) -> usize {
    if address <= CARTRIDGE_ROM_MAIN_BANK_END {
        TYPICAL_ROM_BANK_SIZE * bank0 + (address as usize)
    }
    else {
        TYPICAL_ROM_BANK_SIZE * bank1 + (address as usize & TYPICAL_ROM_ADDRESS_MASK)
    }
}

#[inline(always)]
const fn typical_ram_offset(address: u16, bank: usize) -> usize {
    TYPICAL_RAM_BANK_SIZE * bank + (address as usize & TYPICAL_RAM_ADDRESS_MASK)
}

#[derive(Debug)]
pub enum CartridgeLoadError {
    CannotIdentifyCartridgeType,
    IncorrectMapper { expected: MapperType, actual: MapperType },
    IncorrectROMSize { expected: usize, actual: usize },
    IncorrectRAMSize { expected: usize, actual: usize }
}

impl Display for CartridgeLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CannotIdentifyCartridgeType => f.write_str("Unable to determine the cartridge type"),
            Self::IncorrectMapper { expected, actual } => f.write_fmt(format_args!("Incorrect mapper. Expected {expected:?}, got {actual:?} instead.")),
            Self::IncorrectROMSize { expected, actual } => f.write_fmt(format_args!("Incorrect ROM size. Expected {expected:#08X}, got {actual:#08X} instead.")),
            Self::IncorrectRAMSize { expected, actual } => f.write_fmt(format_args!("Incorrect RAM size. Expected {expected:#08X}, got {actual:#08X} instead."))
        }
    }
}
