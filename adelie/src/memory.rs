//! Memory controller functionality.

/// Trait for memory controllers.
pub trait Memory {
    /// Read a byte at the given address.
    ///
    /// In some cases, the act of reading something can impact the state of the object (also known
    /// as side effects), so this is a mutable function even if it involves reading.
    ///
    /// To read data with no side effects, use [`get_memory`](Memory::get_memory).
    ///
    /// Note that this function is infallible and must return something, even if it is garbage.
    fn read(&mut self, address: u16) -> u8;

    /// Write a byte at the given address.
    ///
    /// Note that, for some memory objects, the act of writing can have side effects.
    ///
    /// To write data with no side effects, use [`get_memory_mut`](Memory::get_memory_mut).
    fn write(&mut self, address: u16, data: u8);

    /// Get the current bank.
    ///
    /// Returns `None` if not implemented.
    fn get_bank(&self) -> Option<usize> {
        None
    }

    /// Get access to the entire memory object.
    ///
    /// Returns `None` if not implemented.
    ///
    /// Depending on how this is implemented, this function might not be available. For example, if
    /// the memory is external (e.g. physical), this may not be available.
    fn get_memory(&self) -> Option<&[u8]> {
        None
    }

    /// Get mutable access to the entire memory object.
    ///
    /// Returns `None` if not implemented.
    ///
    /// Depending on how this is implemented, this function might not be available. For example, if
    /// the memory is external (e.g. physical), this may not be available.
    ///
    /// It is also possible for an object to be read-only. In which case,
    /// [`get_memory`](Memory::get_memory) would return `Some`, but this function wouldn't.
    fn get_memory_mut(&mut self) -> Option<&mut [u8]> {
        None
    }
}

/// Mapped to 0x8000-0x9FFF.
#[derive(Copy, Clone)]
pub struct VideoRAM {
    pub(crate) memory: [u8; 0x4000],
    pub(crate) bank: ByteSize<1>
}

impl VideoRAM {
    fn resolve_address(&self, addr: u16) -> usize {
        let offset = (addr & 0x1FFF) as usize;
        let bank_offset = (self.bank.size & 1) << 13; // * 0x2000
        bank_offset | offset
    }
}

impl Memory for VideoRAM {
    fn read(&mut self, address: u16) -> u8 {
        self.memory[self.resolve_address(address)]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.memory[self.resolve_address(address)] = data
    }

    fn get_bank(&self) -> Option<usize> {
        Some(self.bank.size)
    }

    fn get_memory(&self) -> Option<&[u8]> {
        Some(self.memory.as_slice())
    }

    fn get_memory_mut(&mut self) -> Option<&mut [u8]> {
        Some(self.memory.as_mut_slice())
    }
}

impl Default for VideoRAM {
    fn default() -> Self {
        Self {
            memory: [0u8; 0x4000],
            bank: ByteSize { size: 0 }
        }
    }
}

/// Mapped to 0xC000-0xFDFF.
#[derive(Copy, Clone)]
pub struct WorkRAM {
    memory: [u8; 32768],
    bank: ByteSize<7>
}

impl WorkRAM {
    fn resolve_address(&self, addr: u16) -> usize {
        // Only care about the lower 12 bits.
        let offset = (addr & 0xFFF) as usize;

        // If the upper bit is not set, we are accessing 0xC000-0xCFFF or 0xE000-0xEFFF.
        let bank_offset = if (addr & 0x1000) == 0 {
            // 0xC000-0xCFFF = bank 0
            0
        }
        // Otherwise, we are accessing 0xD000-0xDFFF or 0xF000-0xFDFF.
        else {
            // 0xD000-0xDFFF = bank n
            (self.bank.size & 7) << 12 // * 0x1000
        };

        bank_offset | offset
    }
}

impl Memory for WorkRAM {
    fn read(&mut self, address: u16) -> u8 {
        self.memory[self.resolve_address(address)]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.memory[self.resolve_address(address)] = data
    }

    fn get_bank(&self) -> Option<usize> {
        Some(self.bank.size)
    }

    fn get_memory(&self) -> Option<&[u8]> {
        Some(self.memory.as_slice())
    }

    fn get_memory_mut(&mut self) -> Option<&mut [u8]> {
        Some(self.memory.as_mut_slice())
    }
}

impl Default for WorkRAM {
    fn default() -> Self {
        Self {
            memory: [0u8; 32768],
            bank: ByteSize { size: 1 }
        }
    }
}

/// Mapped to 0xFE00-0xFE9F.
#[derive(Copy, Clone)]
pub struct OAM {
    memory: [u8; 0x100], // have as 0x100 instead of 0xA0 and just do debug checks to prevent generating panic code
}

impl OAM {
    #[inline(always)]
    fn resolve_address(&self, address: u16) -> usize {
        debug_assert!(address >= 0xFE00 && address <= 0xFE9F, "address {address:#04X} is not in OAM");
        (address & 0xFF) as usize
    }
}

impl Memory for OAM {
    fn read(&mut self, address: u16) -> u8 {
        self.memory[self.resolve_address(address)]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.memory[self.resolve_address(address)] = data
    }

    fn get_memory(&self) -> Option<&[u8]> {
        Some(&self.memory.as_slice()[..0xA0])
    }

    fn get_memory_mut(&mut self) -> Option<&mut [u8]> {
        Some(&mut self.memory.as_mut_slice()[..0xA0])
    }
}

impl Default for OAM {
    fn default() -> Self {
        Self {
            memory: [0u8; 0x100]
        }
    }
}

/// Mapped to 0xFF80-0xFFFE
#[derive(Copy, Clone)]
pub struct HighRAM {
    memory: [u8; 0x80] // have as 0x80 instead of 0x7F and just do debug checks to prevent generating panic code
}
impl HighRAM {
    #[inline(always)]
    fn resolve_address(&self, address: u16) -> usize {
        debug_assert!(address >= 0xFF80 && address < 0xFFFF, "address {address:#04X} is not in HRAM");
        (address & 0x7F) as usize
    }
}
impl Memory for HighRAM {
    fn read(&mut self, address: u16) -> u8 {
        self.memory[self.resolve_address(address)]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.memory[self.resolve_address(address)] = data
    }

    fn get_memory(&self) -> Option<&[u8]> {
        Some(&self.memory[..0x7F])
    }

    fn get_memory_mut(&mut self) -> Option<&mut [u8]> {
        Some(&mut self.memory[..0x7F])
    }
}
impl Default for HighRAM {
    fn default() -> Self {
        Self {
            memory: [0u8; 0x80]
        }
    }
}

/// Mapped to 0xFEA0-0xFEFF.
#[derive(Copy, Clone, Default)]
pub struct NullMemory;
impl Memory for NullMemory {
    fn read(&mut self, _address: u16) -> u8 {
        0x00
    }

    fn write(&mut self, _address: u16, _data: u8) {}
}

const BOOT_ROM_LOW_SIZE: usize = 256;
const BOOT_ROM_HIGH_SIZE: usize = 1792;
const BOOT_ROM_PADDING: usize = 512; // used for the ROM

pub type DMGBootROM = [u8; BOOT_ROM_LOW_SIZE];
pub type CGBBootROMPadded = [u8; BOOT_ROM_LOW_SIZE + BOOT_ROM_PADDING + BOOT_ROM_HIGH_SIZE];
pub type CGBBootROM = [u8; BOOT_ROM_LOW_SIZE + BOOT_ROM_HIGH_SIZE];

/// Mapped
#[derive(Copy, Clone)]
pub struct BootROM {
    data: [u8; BOOT_ROM_LOW_SIZE + BOOT_ROM_HIGH_SIZE]
}

impl BootROM {
    pub fn new_dmg(data: DMGBootROM) -> Self {
        Self::from_low_high(data, [0u8; BOOT_ROM_HIGH_SIZE])
    }
    pub fn new_cgb_padded(data: CGBBootROMPadded) -> Self {
        let low: [u8; BOOT_ROM_LOW_SIZE] = data[0..BOOT_ROM_LOW_SIZE].try_into().unwrap();
        let high: [u8; BOOT_ROM_HIGH_SIZE] = data[BOOT_ROM_LOW_SIZE + BOOT_ROM_PADDING..].try_into().unwrap();
        Self::from_low_high(low, high)
    }
    pub fn new_cgb(data: CGBBootROM) -> Self {
        Self { data }
    }
    fn from_low_high(low: [u8; BOOT_ROM_LOW_SIZE], high: [u8; BOOT_ROM_HIGH_SIZE]) -> Self {
        let mut data = [0u8; BOOT_ROM_LOW_SIZE + BOOT_ROM_HIGH_SIZE];
        data[..BOOT_ROM_LOW_SIZE].copy_from_slice(low.as_slice());
        data[BOOT_ROM_LOW_SIZE..].copy_from_slice(high.as_slice());
        Self {
            data
        }
    }
}

impl Default for BootROM {
    fn default() -> Self {
        Self {
            data: [0u8; BOOT_ROM_LOW_SIZE + BOOT_ROM_HIGH_SIZE]
        }
    }
}

impl Memory for BootROM {
    fn read(&mut self, address: u16) -> u8 {
        let address = address as usize;
        let (low, high) = self.data.split_at(BOOT_ROM_LOW_SIZE);

        if address < BOOT_ROM_LOW_SIZE {
            low[address]
        }
        else if address >= 0x200 && address <= 0x8FF {
            high[address - 0x200]
        }
        else {
            debug_assert!(false, "{address:#04X} is not a valid address for a boot ROM");
            0xFF
        }
    }

    fn write(&mut self, _address: u16, _data: u8) {}

    fn get_memory(&self) -> Option<&[u8]> {
        Some(self.data.as_slice())
    }

    fn get_memory_mut(&mut self) -> Option<&mut [u8]> {
        Some(self.data.as_mut_slice())
    }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub(crate) struct ByteSize<const MASK: usize> {
    pub size: usize
}

impl<const MASK: usize> Memory for ByteSize<MASK> {
    fn read(&mut self, _address: u16) -> u8 {
        self.size as u8
    }

    fn write(&mut self, _address: u16, data: u8) {
        self.size = (data as usize) & MASK
    }
}
