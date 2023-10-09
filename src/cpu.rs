use crate::Wrappable;

const R0: usize = 0;
const R1: usize = 1;
const R2: usize = 2;
const R3: usize = 3;

const A1: usize = R0;
const A2: usize = R1;
const A3: usize = R2;
const A4: usize = R3;

const R4: usize = 4;
const R5: usize = 5;
const R6: usize = 6;
const R7: usize = 7;
const R8: usize = 8;

const R9: usize = 9;
const SB: usize = R9;

const R10: usize = 10;

const R11: usize = 11;
const FP: usize = R11;

const V1: usize = R4;
const V2: usize = R5;
const V3: usize = R6;
const V4: usize = R7;
const V5: usize = R8;
const V6: usize = R9;
const V7: usize = R10;
const V8: usize = R11;

const R12: usize = 12;
const IP: usize = R12;

const R13: usize = 13;
const SP: usize = R13;
const R14: usize = 14;
const LR: usize = R14;
const R15: usize = 15;
const PC: usize = R15;

const CPSR: usize = 16;
const SPSR: usize = 17;

const REG_MIN: usize = R0;
const REG_MAX: usize = SPSR;

#[repr(u8)]
#[derive(PartialEq)]

pub enum CPUMode {
    Usr = 0b10000,
    Fiq = 0b10001,
    Irq = 0b10010,
    Svc = 0b10011,
    Abt = 0b10111,
    Und = 0b11011,
    Sys = 0b11111,
}

impl From<u32> for CPUMode {
    fn from(val: u32) -> Self {
        match val {
            i if i as u8 == Self::Usr as u8 => Self::Usr,
            i if i as u8 == Self::Fiq as u8 => Self::Fiq,
            i if i as u8 == Self::Irq as u8 => Self::Irq,
            i if i as u8 == Self::Svc as u8 => Self::Svc,
            i if i as u8 == Self::Abt as u8 => Self::Abt,
            i if i as u8 == Self::Sys as u8 => Self::Sys,
            _ => Self::Und
        }
    }
}

impl From<CPUMode> for usize {
    fn from(value: CPUMode) -> Self {
        match value {
            CPUMode::Usr | CPUMode::Sys => 0,
            CPUMode::Fiq => 1,
            CPUMode::Svc => 2,
            CPUMode::Abt => 3,
            CPUMode::Irq => 4,
            CPUMode::Und => 5
        }
    }
}

pub trait PSR : Register {
    // condition flags
    fn n(&self) -> bool;
    fn z(&self) -> bool;
    fn c(&self) -> bool;
    fn v(&self) -> bool;

    // interrupt flags
    fn i(&self) -> bool;
    fn f(&self) -> bool;

    // thumb flag
    fn t(&self) -> bool;
    fn is_thumb(&self) -> bool { self.t() }
    fn mode(&self) -> CPUMode;

}

impl PSR for u32 {
    // Condition flags
    fn n(&self) -> bool { (self & 0b10000000_00000000_00000000_00000000) != 0 }
    fn z(&self) -> bool { (self & 0b01000000_00000000_00000000_00000000) != 0 }
    fn c(&self) -> bool { (self & 0b00100000_00000000_00000000_00000000) != 0 }
    fn v(&self) -> bool { (self & 0b00010000_00000000_00000000_00000000) != 0 }

    // Interrupt flags
    fn i(&self) -> bool { (self & 0b00000000_00000000_00000000_10000000) != 0 }
    fn f(&self) -> bool { (self & 0b00000000_00000000_00000000_01000000) != 0 }

    fn t(&self) -> bool { (self & 0b00000000_00000000_00000000_00100000) != 0 }
    fn mode(&self) -> CPUMode { (self & 0b00000000_00000000_00000000_00011111).into() }
}



pub trait Register {}

impl Register for u32 {}

#[repr(C)]
pub struct RegisterState {
    pub psr: [u32; 6],
    // general purpose registers
    pub arm_thumb_registers: [u32; 8],
    pub arm_registers: [[u32; 2]; 5],
    pub stack_pointer: [u32; 6],
    pub link_register: [u32; 6],
    pub prgm_counter:  u32,
}

impl RegisterState {
    pub fn get_reg_for_mode(&self, register: usize, mode: CPUMode) -> Option<impl Register> {
        match (register, mode) {
            (register @ 0..=7, _) => self.arm_thumb_registers[register].wrap_some(),
            (register @ 8..=12, mode) if mode == CPUMode::Fiq => self.arm_registers[register - 8][usize::from(mode)].wrap_some(),
            (register @ 8..=12, _) => self.arm_registers[register - 8][0].wrap_some(),
            (13, mode) => self.stack_pointer[usize::from(mode)].wrap_some(),
            (14, mode) => self.link_register[usize::from(mode)].wrap_some(),
            (15, _) => self.prgm_counter.wrap_some(),
            (16, _) => self.psr[0].wrap_some(),
            (17, mode) => self.psr[usize::from(mode)].wrap_some(),
            _ => None
        }
    }

    pub fn set_reg_for_mode(mut self, register: usize, mode: CPUMode, value: u32) -> Self {
        if !(REG_MIN..=REG_MAX).contains(&register) {
            return self
        }

        match (register, mode) {
            (register @ R0..=R7, _) => self.arm_thumb_registers[register] = value,
            (register @ R8..=R12, mode) if mode == CPUMode::Fiq => self.arm_registers[register - 8][usize::from(mode)] = value,
            (register @ R8..=R12, _) => self.arm_registers[register - 8][0] = value,
            (SP, mode) => self.stack_pointer[usize::from(mode)] = value,
            (LR, mode) => self.link_register[usize::from(mode)] = value,
            (PC, _) => self.prgm_counter = value,
            (CPSR, _) => self.psr[0] = value,
            // TODO: Check if this could cause trouble.
            (SPSR, mode) => self.psr[usize::from(mode)] = value,
            (_, _) => {}
        };

        self
    }
}

struct Arm7TDMI {
    registers: RegisterState,
}

const KB: usize = 1024;
const MB: usize = 1024 * KB;

struct GBAMmio {
    GBABios:    [u8; 016 * KB],         // 0x00000000..=0x00003FFF
    WRAMFast:   [u8; 032 * KB],         // 0x03000000..=0x03007FFF
    WRAMSlow:   [u8; 256 * KB],         // 0x02000000..=0x0203FFFF
    IO:         [u8; 0x0003FE],         // 0x04000000..=0x040003FE
    PALETTE:    [u8; 001 * KB],         // 0x05000000..=0x050003FF
    VRAM:       [u8; 096 * KB],         // 0x06000000..=0x06017FFF
    OAM:        [u8; 001 * KB],         // 0x07000000..=0x070003FF
    ExternMem:  [u8; 032 * KB],         // 0x08000000..=0x09FFFFFF | 0x0A000000..=0BFFFFFF | 0x0C000000..=0DFFFFFF
    ExternSRAM: [u8; 064 * KB],         // 0x0E000000..=0x0E00FFFF
}



macro_rules! translate {
    ($addr:expr, $size: expr, $(($start:literal, $end:literal))*) => {
        match $addr {
            $($start..=$end => {
                let end = $start + $size;
                if end > $end {
                    None
                } else {
                    let start = $addr - $start;
                    let end = end - $start;
                    (start, end).wrap_some()
                }
            })*
            _ => None,
        }
    }
}

impl GBAMmio {
    fn translate_address(address: usize, size: usize) -> Option<(usize, usize)> {
            translate!(address, size, 
                (0x00000000, 0x00003FFF) 
                (0x02000000, 0x0203FFFF) 
                (0x03000000, 0x03007FFF)
                (0x04000000, 0x040003FE)
                (0x05000000, 0x050003FF)
                (0x06000000, 0x06017FFF)
                (0x07000000, 0x070003FF)
                (0x08000000, 0x09FFFFFF)
                (0x0A000000, 0x0BFFFFFF)
                (0x0C000000, 0x0DFFFFFF)
                (0x0E000000, 0x0E00FFFF)
            )
    }

    pub fn read(&self, address: usize, n_bytes: usize) -> Option<&[u8]> {
        if let Some((address, end_address)) = Self::translate_address(address, n_bytes) {

            match address {
                0x00000000..=0x00003FFF => self.GBABios    [address..=end_address].wrap_some(),
                0x02000000..=0x0203FFFF => self.WRAMSlow   [address..=end_address].wrap_some(),
                0x03000000..=0x03007FFF => self.WRAMFast   [address..=end_address].wrap_some(),
                0x04000000..=0x040003FE => self.IO         [address..=end_address].wrap_some(),
                0x05000000..=0x050003FF => self.PALETTE    [address..=end_address].wrap_some(),
                0x06000000..=0x06017FFF => self.VRAM       [address..=end_address].wrap_some(),
                0x07000000..=0x070003FF => self.OAM        [address..=end_address].wrap_some(),
                0x08000000..=0x09FFFFFF | 
                0x0A000000..=0x0BFFFFFF | 
                0x0C000000..=0x0DFFFFFF => self.ExternMem  [address..=end_address].wrap_some(),
                0x0E000000..=0x0E00FFFF => self.ExternSRAM [address..=end_address].wrap_some(),
                _                       => None
            }

        } else { None }
    }

    pub fn write(&mut self, address: usize, bytes: &[u8]) -> bool {
        if let Some((address, end_address)) = Self::translate_address(address, bytes.len()) {
            match address {
                0x00000000..=0x00003FFF => self.GBABios    [address..=end_address].copy_from_slice(bytes),
                0x02000000..=0x0203FFFF => self.WRAMSlow   [address..=end_address].copy_from_slice(bytes),
                0x03000000..=0x03007FFF => self.WRAMFast   [address..=end_address].copy_from_slice(bytes),
                0x04000000..=0x040003FE => self.IO         [address..=end_address].copy_from_slice(bytes),
                0x05000000..=0x050003FF => self.PALETTE    [address..=end_address].copy_from_slice(bytes),
                0x06000000..=0x06017FFF => self.VRAM       [address..=end_address].copy_from_slice(bytes),
                0x07000000..=0x070003FF => self.OAM        [address..=end_address].copy_from_slice(bytes),
                0x08000000..=0x09FFFFFF | 
                0x0A000000..=0x0BFFFFFF | 
                0x0C000000..=0x0DFFFFFF => self.ExternMem  [address..=end_address].copy_from_slice(bytes),
                0x0E000000..=0x0E00FFFF => self.ExternSRAM [address..=end_address].copy_from_slice(bytes),
                _                       => {}
            }
            true
        } else {
            false
        }        
    }

    /*
  4000000h  2    R/W  DISPCNT   LCD Control
  4000002h  2    R/W  -         Undocumented - Green Swap
  4000004h  2    R/W  DISPSTAT  General LCD Status (STAT,LYC)
  4000006h  2    R    VCOUNT    Vertical Counter (LY)
  4000008h  2    R/W  BG0CNT    BG0 Control
  400000Ah  2    R/W  BG1CNT    BG1 Control
  400000Ch  2    R/W  BG2CNT    BG2 Control
  400000Eh  2    R/W  BG3CNT    BG3 Control
  4000010h  2    W    BG0HOFS   BG0 X-Offset
  4000012h  2    W    BG0VOFS   BG0 Y-Offset
  4000014h  2    W    BG1HOFS   BG1 X-Offset
  4000016h  2    W    BG1VOFS   BG1 Y-Offset
  4000018h  2    W    BG2HOFS   BG2 X-Offset
  400001Ah  2    W    BG2VOFS   BG2 Y-Offset
  400001Ch  2    W    BG3HOFS   BG3 X-Offset
  400001Eh  2    W    BG3VOFS   BG3 Y-Offset
  4000020h  2    W    BG2PA     BG2 Rotation/Scaling Parameter A (dx)
  4000022h  2    W    BG2PB     BG2 Rotation/Scaling Parameter B (dmx)
  4000024h  2    W    BG2PC     BG2 Rotation/Scaling Parameter C (dy)
  4000026h  2    W    BG2PD     BG2 Rotation/Scaling Parameter D (dmy)
  4000028h  4    W    BG2X      BG2 Reference Point X-Coordinate
  400002Ch  4    W    BG2Y      BG2 Reference Point Y-Coordinate
  4000030h  2    W    BG3PA     BG3 Rotation/Scaling Parameter A (dx)
  4000032h  2    W    BG3PB     BG3 Rotation/Scaling Parameter B (dmx)
  4000034h  2    W    BG3PC     BG3 Rotation/Scaling Parameter C (dy)
  4000036h  2    W    BG3PD     BG3 Rotation/Scaling Parameter D (dmy)
  4000038h  4    W    BG3X      BG3 Reference Point X-Coordinate
  400003Ch  4    W    BG3Y      BG3 Reference Point Y-Coordinate
  4000040h  2    W    WIN0H     Window 0 Horizontal Dimensions
  4000042h  2    W    WIN1H     Window 1 Horizontal Dimensions
  4000044h  2    W    WIN0V     Window 0 Vertical Dimensions
  4000046h  2    W    WIN1V     Window 1 Vertical Dimensions
  4000048h  2    R/W  WININ     Inside of Window 0 and 1
  400004Ah  2    R/W  WINOUT    Inside of OBJ Window & Outside of Windows
  400004Ch  2    W    MOSAIC    Mosaic Size
  400004Eh       -    -         Not used
  4000050h  2    R/W  BLDCNT    Color Special Effects Selection
  4000052h  2    R/W  BLDALPHA  Alpha Blending Coefficients
  4000054h  2    W    BLDY      Brightness (Fade-In/Out) Coefficient
  4000056h       -    -         Not used
     */
    
}