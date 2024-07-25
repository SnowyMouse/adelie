use crate::cartridge::Cartridge;
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
pub struct Emulator<Cart: Cartridge, U: Sized> {
    user_data: U,
    soc_clock_high: bool,
    soc_clock: u32,
    io: IO<Cart>,

    #[cfg(feature = "std")]
    clock: Clock,
    #[cfg(feature = "std")]
    last_clock_count: u64
}

const SOC_BASE_CLOCK_SPEED: u32 = 1024 * 1024 * 4;
const SOC_BASE_CLOCK_SPEED_DOUBLE_SPEED: u32 = SOC_BASE_CLOCK_SPEED *2;

impl<Cart: Cartridge, U: Sized> Emulator<Cart, U> {
    pub fn new(
        user_data: U,
        cartridge: Cart,
        boot_rom: BootROM,
        model: Model
    ) -> Self {
        Self {
            user_data,
            soc_clock_high: false,
            soc_clock: 0,
            io: IO {
                double_speed_mode: false,
                cartridge,
                boot_rom: BufferedInstantMemory::new(boot_rom),
                video_ram: Default::default(),
                work_ram: Default::default(),
                oam: Default::default(),
                high_ram: Default::default(),
                no_access: NullMemory::default(),
                model,
                registers: IORegisters::default(),
            },
            clock: Clock::new(),
            last_clock_count: 0
        }
    }

    /// Get the user data.
    pub fn get_user_data(&self) -> &U {
        &self.user_data
    }

    /// Get a mutable reference to the user data.
    pub fn get_user_data_mut(&mut self) -> &mut U {
        &mut self.user_data
    }

    /// Destroy the instance to get the user data back.
    pub fn into_user_data(self) -> U {
        self.user_data
    }

    /// Run half of one SoC clock cycle.
    ///
    /// This function is untimed and must be called at 4x2 MiHz or 8x2 MiHz depending on if running
    /// in double speed mode, alternating between high (true) and low (false).
    ///
    /// Calling this function with the same signal as last time is a no-op.
    pub fn tick_soc(&mut self, high: bool) {
        if self.soc_clock_high == high {
            // do nothing
            return;
        }
        self.soc_clock_high = high;
        todo!("tick_soc")
    }

    /// Run the SoC timed.
    ///
    /// This will try to yield to the OS scheduler when possible, which may sometimes be less
    /// accurate for timing but more efficient.
    #[cfg(feature = "std")]
    pub fn tick_soc_timed(&mut self) {
        let doubled = self.soc_clock_speed()*2;
        while !self.tick_soc_if_ready(doubled) {
            std::thread::yield_now();
        }
    }

    /// Run the SoC timed.
    ///
    /// This uses busy waiting which will burn the CPU instead of yielding, which may be more
    /// accurate for timing in some scenarios but considerably less efficient.
    #[cfg(feature = "std")]
    pub fn tick_soc_timed_busywait(&mut self) {
        let doubled = self.soc_clock_speed()*2;
        while !self.tick_soc_if_ready(doubled) {}
    }

    /// Run one M-cycle with the given speed multiplier.
    ///
    /// This is slightly more efficient than ticking the SoC on/off manually, but less accurate if
    /// going for cycle accuracy.
    ///
    /// This will try to yield to the OS scheduler when possible, which may sometimes be less
    /// accurate for timing but more efficient.
    #[cfg(feature = "std")]
    pub fn m_cycle_soc(&mut self, speed: f64) {
        let clock_speed = (((self.soc_clock_speed()/4) as f64) * speed) as u32;
        while !self.tick_soc_if_ready(clock_speed) {
            std::thread::yield_now();
        }
        for _ in 0..4 {
            self.tick_soc(true);
            self.tick_soc(false);
        }
    }

    /// Run one M-cycle with the given speed multiplier.
    ///
    /// This is slightly more efficient than ticking the SoC on/off manually, but less accurate if
    /// going for cycle accuracy.
    ///
    /// This uses busy waiting which will burn the CPU instead of yielding, which may be more
    /// accurate for timing in some scenarios but considerably less efficient.
    #[cfg(feature = "std")]
    pub fn m_cycle_soc_busywait(&mut self, speed: f64) {
        let clock_speed = (((self.soc_clock_speed()/4) as f64) * speed) as u32;
        while !self.tick_soc_if_ready(clock_speed) {}
        for _ in 0..4 {
            self.tick_soc(true);
            self.tick_soc(false);
        }
    }

    /// Return the clock speed of the SoC in Hz.
    pub const fn soc_clock_speed(&self) -> u32 {
        if self.in_double_speed_mode() {
            SOC_BASE_CLOCK_SPEED_DOUBLE_SPEED
        }
        else {
            SOC_BASE_CLOCK_SPEED
        }
    }

    /// Get whether or not the console is running in double speed mode.
    #[inline(always)]
    pub const fn in_double_speed_mode(&self) -> bool {
        self.io.double_speed_mode
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

    #[cfg(feature = "std")]
    fn tick_soc_if_ready(&mut self, clock_speed: u32) -> bool {
        let total_clocks = self.clock.total_clocks(clock_speed);
        if total_clocks <= self.last_clock_count {
            std::thread::yield_now();
            return false;
        }
        self.last_clock_count = total_clocks;
        self.tick_soc(!self.soc_clock_high);
        true
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

/// 4/8 MiHz clock
#[cfg(feature = "std")]
#[derive(Copy, Clone)]
struct Clock {
    start_time: std::time::Instant
}

#[cfg(feature = "std")]
impl Clock {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now()
        }
    }
    pub fn total_clocks(&self, speed: u32) -> u64 {
        let speed = speed as u128;
        let time_since_start = (std::time::Instant::now() - self.start_time).as_nanos();
        (time_since_start * speed / 1000000000) as u64
    }
}
