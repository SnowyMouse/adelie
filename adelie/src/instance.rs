use crate::instance::io::{IO, IORegisters};
use crate::memory::{BootROM, BufferedInstantMemory, InstantMemory, Memory, NullMemory};

pub(crate) mod io;

#[derive(Copy, Clone)]
pub enum Model {
    DMG,
    CGB
}
impl Model {
    pub const fn is_dmg(self) -> bool {
        match self {
            Self::DMG => true,
            Self::CGB => false
        }
    }
    pub const fn is_cgb(self) -> bool {
        match self {
            Self::DMG => false,
            Self::CGB => true
        }
    }
}

#[derive(Copy, Clone)]
pub struct Emulator<Cart: Memory> {
    soc_clock: u32,
    io: IO<Cart>
}

const SOC_BASE_CLOCK_SPEED: u32 = 1024 * 1024 * 4;

impl<Cart: Memory> Emulator<Cart> {
    pub fn new(
        cartridge: Cart,
        boot_rom: BootROM,
        model: Model
    ) -> Self {
        Self {
            soc_clock: 0,
            io: IO {
                cartridge,
                boot_rom: BufferedInstantMemory::new(boot_rom),
                video_ram: Default::default(),
                work_ram: Default::default(),
                oam: Default::default(),
                high_ram: Default::default(),
                no_access: NullMemory::default(),
                model,
                registers: IORegisters::default(),
            }
        }
    }

    /// Run one SoC clock cycle.
    pub fn tick_soc(&mut self) {
        todo!()
    }

    /// Access the internal memory of the given memory type.
    ///
    /// This is primarily available for debugging.
    pub fn get_internal_memory_mut(&mut self, memory_type: InstantMemoryType) -> &mut dyn InstantMemory {
        match memory_type {
            InstantMemoryType::WRAM => &mut self.io.work_ram.memory,
            InstantMemoryType::VRAM => &mut self.io.video_ram.memory,
            InstantMemoryType::BootROM => &mut self.io.boot_rom.memory,
            InstantMemoryType::OAM => &mut self.io.oam.memory,
            InstantMemoryType::HRAM => &mut self.io.high_ram.memory,
        }
    }

    /// Access the internal memory of the cartridge.
    pub fn get_cartridge_mut(&mut self) -> &mut Cart {
        &mut self.io.cartridge
    }

    /// Get the current emulated model of this instance.
    pub fn get_model(&self) -> Model {
        self.io.model
    }
}

pub enum InstantMemoryType {
    WRAM,
    VRAM,
    BootROM,
    OAM,
    HRAM
}

#[derive(Copy, Clone, Default)]
pub struct StubbedInterface<const STATIC_VALUE: u8>;
impl<const STATIC_VALUE: u8> Memory for StubbedInterface<STATIC_VALUE> {
    fn set_data_lines(&mut self, _address: u16, _write: bool, _data_in: u8) {}

    fn read_out(&mut self) -> u8 {
        STATIC_VALUE
    }
}
