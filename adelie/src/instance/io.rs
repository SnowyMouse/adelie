use crate::cartridge::Cartridge;
use crate::instance::{Model, SOC_BASE_CLOCK_SPEED, StubbedInterface};
use crate::memory::{BootROM, HighRAM, Memory, NullMemory, OAM, VideoRAM, WorkRAM};

#[derive(Copy, Clone)]
pub struct IO<Cart: Cartridge> {
    pub cartridge: Cart,
    pub boot_rom: BootROM,
    pub registers: IORegisters,
    pub video_ram: VideoRAM,
    pub work_ram: WorkRAM,
    pub oam: OAM,
    pub high_ram: HighRAM,
    pub io_data: IORegisters,
    pub no_access: NullMemory,
    pub model: Model
}

#[derive(Copy, Clone, Default)]
pub struct IORegisters {
    pub joypad_data: JoypadData,
    pub serial_transfer: StubbedInterface,
    pub timer_div: TimerDIV,
    pub interrupts: Interrupts,
    pub audio: StubbedInterface,
    pub wave_pattern: StubbedInterface,
    pub lcd: StubbedInterface,
    pub disable_bootrom: DisableBootROM,
    pub vram_dma: StubbedInterface,
    pub bg_obj_palettes: StubbedInterface,
    pub wram_bank_select: StubbedInterface,
    pub prepare_speed_switch: StubbedInterface,
    pub infrared: StubbedInterface,
    pub object_priority: StubbedInterface,
    pub unused: StubbedInterface
}

impl<Cart: Cartridge> IO<Cart> {
    fn resolve_address_to_device(&mut self, address: u16) -> &mut dyn Memory {
        match address {
            0x0000..=0x7FFF => {
                if (self.io_data.disable_bootrom.byte[0] != 0) && (address < 0x100 || (address >= 0x300 && self.model.is_cgb())) {
                    &mut self.boot_rom
                }
                else {
                    &mut self.cartridge
                }
            },
            0x8000..=0x9FFF => &mut self.video_ram,
            0xA000..=0xBFFF => &mut self.cartridge,
            0xC000..=0xFDFF => &mut self.work_ram,
            0xFE00..=0xFE9F => &mut self.oam,
            0xFEA0..=0xFEFF => &mut self.no_access,
            0xFF00..=0xFFFF => {
                match (address & 0xFF) as u8 {
                    // HRAM
                    0x7F..=0xFE => &mut self.high_ram,

                    // DMG and CGB
                    0x00        => &mut self.registers.joypad_data,
                    0x01 | 0x02 => &mut self.registers.serial_transfer,
                    0x04..=0x07 => &mut self.registers.timer_div,
                    0x0F        => &mut self.registers.interrupts,
                    0x10..=0x26 => &mut self.registers.audio,
                    0x27..=0x2F => &mut self.registers.unused,
                    0x30..=0x3F => &mut self.registers.wave_pattern,
                    0x40..=0x4B => &mut self.registers.lcd,
                    0x50        => &mut self.registers.disable_bootrom,
                    0xFF        => &mut self.registers.interrupts,

                    // Unused regardless
                    0x03        => &mut self.registers.unused,
                    0x08..=0x0E => &mut self.registers.unused,
                    0x4C        => &mut self.registers.unused,
                    0x4E        => &mut self.registers.unused,
                    0x57..=0x67 => &mut self.registers.unused,
                    0x6D..=0x6F => &mut self.registers.unused,
                    0x71..=0x7E => &mut self.registers.unused,

                    // all registers below are CGB exclusive
                    _ if !self.model.is_cgb() => &mut self.registers.unused,

                    // CGB only
                    0x4D        => &mut self.registers.prepare_speed_switch,
                    0x4F        => &mut self.video_ram,
                    0x51..=0x55 => &mut self.registers.vram_dma,
                    0x56        => &mut self.registers.infrared,
                    0x68..=0x6B => &mut self.registers.bg_obj_palettes,
                    0x6C        => &mut self.registers.object_priority,
                    0x70        => &mut self.registers.wram_bank_select,
                }
            }
        }
    }
}

impl<Cart: Cartridge> Memory for IO<Cart> {
    fn read(&mut self, address: u16) -> u8 {
        self.resolve_address_to_device(address).read(address)
    }

    fn write(&mut self, address: u16, data: u8) {
        self.resolve_address_to_device(address).write(address, data)
    }
}

#[derive(Copy, Clone, Default)]
pub struct JoypadData {
    pub select_buttons: bool,
    pub select_dpad: bool,
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl Memory for JoypadData {
    fn read(&mut self, _address: u16) -> u8 {
        let mut result = 0b111111;

        if self.select_dpad {
            result &= !0b10000;
            result &= !((self.right as u8) << 0);
            result &= !((self.left as u8) << 1);
            result &= !((self.up as u8) << 2);
            result &= !((self.down as u8) << 3);
        }

        if self.select_buttons {
            result &= !0b100000;
            result &= !((self.a as u8) << 0);
            result &= !((self.b as u8) << 1);
            result &= !((self.select as u8) << 2);
            result &= !((self.start as u8) << 3);
        }

        result
    }

    fn write(&mut self, _address: u16, data: u8) {
        let data = data >> 4;
        self.select_dpad = data & 1 != 0;
        self.select_buttons = data & 2 != 0;
    }
}

#[derive(Copy, Clone)]
pub struct DisableBootROM {
    pub byte: [u8; 1]
}
impl Default for DisableBootROM {
    fn default() -> Self {
        Self { byte: [0; 1] }
    }
}

impl Memory for DisableBootROM {
    fn read(&mut self, _address: u16) -> u8 {
        self.byte[0]
    }
    fn write(&mut self, _address: u16, data: u8) {
        if self.byte[0] != 0 {
            self.byte[0] = data
        }
    }
    fn get_memory(&self) -> Option<&[u8]> {
        Some(&self.byte)
    }
    fn get_memory_mut(&mut self) -> Option<&mut [u8]> {
        Some(&mut self.byte)
    }
}

#[derive(Copy, Clone)]
pub struct TimerDIV {
    value: [u8; 4]
}
impl TimerDIV {
    pub(crate) fn tick_div(&mut self, soc_clock_count: u32) {
        let tick_div = soc_clock_count % (SOC_BASE_CLOCK_SPEED / 16384);
        if tick_div == 0 {
            let div = self.get_timer_counter();
            *div = div.wrapping_add(1);
        }
    }
    pub(crate) fn tick_timer(&mut self, soc_clock_count: u32) -> bool {
        let control = *self.get_timer_control();
        if (control & 0b100) != 0 {
            let rate = match control & 0b11 {
                0 => 4096,
                1 => 262144,
                2 => 65536,
                4 => 16384,
                _ => unreachable!()
            };
            let tick_tima = soc_clock_count % (SOC_BASE_CLOCK_SPEED / rate);
            if tick_tima == 0 {
                let modulo = *self.get_timer_modulo();
                let c = self.get_timer_counter();
                let (new_c, overflowed) = c.overflowing_add(1);
                *c = new_c;
                if overflowed {
                    *c = modulo;
                }
                return overflowed;
            }
        }
        false
    }
    pub fn get_div(&mut self) -> &mut u8 {
        &mut self.value[0]
    }
    pub fn get_timer_counter(&mut self) -> &mut u8 {
        &mut self.value[1]
    }
    pub fn get_timer_modulo(&mut self) -> &mut u8 {
        &mut self.value[2]
    }
    pub fn get_timer_control(&mut self) -> &mut u8 {
        &mut self.value[3]
    }
}
impl Default for TimerDIV {
    fn default() -> Self {
        Self { value: [0,0,0,0] }
    }
}
impl Memory for TimerDIV {
    fn read(&mut self, address: u16) -> u8 {
        match address & 3 {
            0 => *self.get_div(),           // DIV
            1 => *self.get_timer_counter(), // TIMA
            2 => *self.get_timer_modulo(),  // TMA
            3 => *self.get_timer_control(), // TAC
            _ => unreachable!()
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address & 3 {
            0 => *self.get_div() = 0,              // DIV
            1 => (),                               // TIMA
            2 => *self.get_timer_modulo() = data,  // TMA
            3 => *self.get_timer_control() = data, // TAC
            _ => unreachable!()
        }
    }

    fn get_memory(&self) -> Option<&[u8]> {
        Some(self.value.as_slice())
    }

    fn get_memory_mut(&mut self) -> Option<&mut [u8]> {
        Some(self.value.as_mut_slice())
    }
}

#[derive(Copy, Clone, Default)]
pub struct Interrupts {
    pub interrupt_enabled: u8,
    pub interrupt_requested: u8
}

impl Interrupts {
    fn resolve_address_to_byte(&mut self, address: u16) -> &mut u8 {
        if (address & 0xF0) == 0 {
            &mut self.interrupt_requested
        }
        else {
            &mut self.interrupt_enabled
        }
    }
}

impl Memory for Interrupts {
    fn read(&mut self, address: u16) -> u8 {
        *self.resolve_address_to_byte(address)
    }

    fn write(&mut self, address: u16, data: u8) {
        *self.resolve_address_to_byte(address) = data
    }
}
