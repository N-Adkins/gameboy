use crate::emulator::bus::Bus;

#[derive(Debug)]
enum Reg8 {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
    HLInd, // Indexing memory with HL for a byte
}

impl Reg8 {
    fn from_bits(bits: u8) -> Reg8 {
        match bits & 0b111 {
            0 => Self::B,
            1 => Self::C,
            2 => Self::D,
            3 => Self::E,
            4 => Self::H,
            5 => Self::L,
            6 => Self::HLInd,
            7 => Self::A,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
enum Reg16 {
    AF,
    BC,
    DE,
    HL,
}

#[derive(Debug)]
#[repr(transparent)]
struct Flags(u8);

impl Flags {
    fn z(self) -> bool {
        self.0 & 0x80 != 0
    }
    
    fn n(self) -> bool {
        self.0 & 0x40 != 0
    }

    fn h(self) -> bool {
        self.0 & 0x20 != 0
    }

    fn c(self) -> bool {
        self.0 & 0x10 != 0
    }

    fn set_z(&mut self, v: bool) {
        self.set_bit(0x80, v);
    }

    fn set_n(&mut self, v: bool) {
        self.set_bit(0x40, v);
    }

    fn set_h(&mut self, v: bool) {
        self.set_bit(0x20, v);
    }

    fn set_c(&mut self, v: bool) {
        self.set_bit(0x10, v);
    }

    fn set_bit(&mut self, mask: u8, v: bool) {
        if v {
            self.0 |= mask
        } else {
            self.0 &= !mask
        }
    }
}

#[derive(Debug)]
struct Cpu {
    a: u8,
    f: Flags,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
}

impl Cpu {
    fn step(&mut self, bus: &mut Bus) {
        let op = self.next_byte(bus);

        match op {
            0x40..=0x7F => { // LD R, R
                let dst = Reg8::from_bits(op >> 3);
                let src = Reg8::from_bits(op);
                let value = self.get_reg8(bus, src);
                self.set_reg8(bus, dst, value);
            },
        }
    }

    fn get_reg8(&self, bus: &mut Bus, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.a,
            Reg8::F => self.f.0,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::H => self.h,
            Reg8::L => self.l,
            Reg8::HLInd => bus.read_byte(self.get_reg16(Reg16::HL)),
        }
    }

    fn set_reg8(&mut self, bus: &mut Bus, reg: Reg8, value: u8) {
        match reg {
            Reg8::A => self.a = value,
            Reg8::F => self.f.0 = value & 0xF0,
            Reg8::B => self.b = value,
            Reg8::C => self.c = value,
            Reg8::D => self.d = value,
            Reg8::E => self.e = value,
            Reg8::H => self.h = value,
            Reg8::L => self.l = value,
            Reg8::HLInd => bus.write_byte(self.get_reg16(Reg16::HL), value),
        }
    }

    fn get_reg16(&self, reg: Reg16) -> u16 {
        match reg {
            Reg16::AF => u16::from_be_bytes([self.a, self.f.0]),
            Reg16::BC => u16::from_be_bytes([self.b, self.c]),
            Reg16::DE => u16::from_be_bytes([self.d, self.e]),
            Reg16::HL => u16::from_be_bytes([self.h, self.l]),
        }
    }

    fn set_reg16(&mut self, reg: Reg16, value: u16) {
        let [hi, lo] = u16::to_be_bytes(value);
        match reg {
            Reg16::AF => {
                self.a = hi;
                self.f.0 = lo & 0xF0; // Bottom 4 bits must be 0
            },
            Reg16::BC => {
                self.b = hi;
                self.c = lo;
            },
            Reg16::DE => {
                self.d = hi;
                self.e = lo;
            },
            Reg16::HL => {
                self.h = hi;
                self.l = lo;
            }
        }
    }

    fn next_byte(&mut self, bus: &mut Bus) -> u8 {
        let byte = bus.read_byte(self.pc);
        self.pc += 1;
        byte
    }
}