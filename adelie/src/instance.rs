pub(crate) mod io;

use crate::cartridge::Cartridge;
use crate::instance::io::{IO, IORegisters};
use crate::memory::{BootROM, HighRAM, Memory, NullMemory, OAM, VideoRAM, WorkRAM};

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
pub struct Emulator<Cart: Cartridge> {
    soc_clock: u32,
    io: IO<Cart>
}

const SOC_BASE_CLOCK_SPEED: u32 = 1024 * 1024 * 4;

impl<Cart: Cartridge> Emulator<Cart> {
    pub fn new(
        cartridge: Cart,
        boot_rom: BootROM,
        model: Model
    ) -> Self {
        Self {
            soc_clock: 0,
            io: IO {
                cartridge,
                boot_rom,
                video_ram: VideoRAM::default(),
                work_ram: WorkRAM::default(),
                oam: OAM::default(),
                high_ram: HighRAM::default(),
                io_data: IORegisters::default(),
                no_access: NullMemory::default(),
                model,
                registers: IORegisters::default(),
            }
        }
    }

    /// Run one SoC clock cycle.
    pub fn tick_soc(&mut self) {
        let c = self.soc_clock;
        self.io.io_data.timer_div.tick_div(c);
        if self.io.io_data.timer_div.tick_timer(c) {
            self.io.registers.interrupts.interrupt_requested |= 0b100;
        }

        let next_clock = (c + 1) % SOC_BASE_CLOCK_SPEED;
        self.soc_clock = next_clock;
    }
}

#[derive(Copy, Clone, Default)]
pub struct StubbedInterface<const STATIC_VALUE: u8>;
impl<const STATIC_VALUE: u8> Memory for StubbedInterface<STATIC_VALUE> {
    fn read(&mut self, _address: u16) -> u8 {
        STATIC_VALUE
    }

    fn write(&mut self, _address: u16, _data: u8) {}
}
