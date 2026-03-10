use crate::emulator::bus::Bus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reg16 {
    AF,
    BC,
    DE,
    HL,
}

#[derive(Debug, Clone, Copy)]
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

fn checked_add8(lhs: u8, rhs: u8, old_carry: bool) -> (u8, Flags) {
    let (value, carry) = lhs.carrying_add(rhs, old_carry);
    let half_carry = (lhs & 0xF) + (rhs & 0xF) + (old_carry as u8) > 0xF;
    let zero = value == 0;
    let mut flags = Flags(0);
    flags.set_z(zero);
    flags.set_n(false); // add
    flags.set_c(carry);
    flags.set_h(half_carry);
    (value, flags)
}

fn checked_add16(lhs: u16, rhs: u16, old_carry: bool) -> (u16, Flags) {
    let (value, carry) = lhs.carrying_add(rhs, old_carry);
    let half_carry = (lhs & 0xFF) + (rhs & 0xFF) + (old_carry as u16) > 0xFF;
    let zero = value == 0;
    let mut flags = Flags(0);
    flags.set_z(zero);
    flags.set_n(false); // add
    flags.set_c(carry);
    flags.set_h(half_carry);
    (value, flags)
}

fn checked_sub8(lhs: u8, rhs: u8, old_carry: bool) -> (u8, Flags) {
    let value = lhs - rhs - old_carry as u8;
    let carry = value > lhs;
    let half_carry = (lhs & 0xF) - (rhs & 0xF) - (old_carry as u8) > 0xF;
    let zero = value == 0;
    let mut flags = Flags(0);
    flags.set_z(zero);
    flags.set_n(true); // sub
    flags.set_c(carry);
    flags.set_h(half_carry);
    (value, flags)
}

fn checked_sub16(lhs: u16, rhs: u16, old_carry: bool) -> (u16, Flags) {
    let value = lhs - rhs - old_carry as u16;
    let carry = value > lhs;
    let half_carry = (lhs & 0xFF) - (rhs & 0xFF) - (old_carry as u16) > 0xFF;
    let zero = value == 0;
    let mut flags = Flags(0);
    flags.set_z(zero);
    flags.set_n(true); // sub
    flags.set_c(carry);
    flags.set_h(half_carry);
    (value, flags)
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
        let op = self.next8(bus);

        match op {
            0x00 => {} // NOP

            0x10 => {
                // STOP
                todo!();
            }

            0x20 => {
                // JR NZ, s8
                if self.f.z() {
                    let val = self.next8(bus) as i8 as i16;
                    self.pc = self.pc.wrapping_add_signed(val);
                }
            }

            0x30 => {
                // JR NC, s8
                if self.f.c() {
                    let val = self.next8(bus) as i8 as i16;
                    self.pc = self.pc.wrapping_add_signed(val);
                }
            }

            0x01 => {
                // LD BC, d16
                let value = self.next16(bus);
                self.set_reg16(Reg16::BC, value);
            }

            0x11 => {
                // LD DE, d16
                let value = self.next16(bus);
                self.set_reg16(Reg16::DE, value);
            }

            0x21 => {
                // LD HL, d16
                let value = self.next16(bus);
                self.set_reg16(Reg16::HL, value);
            }

            0x31 => {
                // LD SP, d16
                let value = self.next16(bus);
                self.sp = value;
            }

            0x02 => {
                // LD (BC), A
                let addr = self.get_reg16(Reg16::BC);
                bus.write_byte(addr, self.a);
            }

            0x12 => {
                // LD (DE), A
                let addr = self.get_reg16(Reg16::DE);
                bus.write_byte(addr, self.a);
            }

            0x22 => {
                // LD (HL+), A
                let addr = self.get_reg16(Reg16::HL);
                bus.write_byte(addr, self.a);
                self.set_reg16(Reg16::HL, addr + 1);
            }

            0x32 => {
                // LD (HL-), A
                let addr = self.get_reg16(Reg16::HL);
                bus.write_byte(addr, self.a);
                self.set_reg16(Reg16::HL, addr - 1);
            }

            0x03 => {
                // INC BC
                let value = self.get_reg16(Reg16::BC) + 1;
                self.set_reg16(Reg16::BC, value);
            }

            0x13 => {
                // INC DE
                let value = self.get_reg16(Reg16::DE) + 1;
                self.set_reg16(Reg16::DE, value);
            }

            0x23 => {
                // INC HL
                let value = self.get_reg16(Reg16::HL) + 1;
                self.set_reg16(Reg16::HL, value);
            }

            0x33 => {
                // INC SP
                self.sp += 1;
            }

            0x04 => {
                // INC B
                let h = (self.b & 0xF) == 0xF;
                self.f.set_z(self.b == 0);
                self.f.set_n(false);
                self.f.set_h(h);
            }

            0x14 => {
                // INC D
                let h = (self.d & 0xF) == 0xF;
                self.f.set_z(self.d == 0);
                self.f.set_n(false);
                self.f.set_h(h);
            }

            0x24 => {
                // INC H
                let h = (self.h & 0xF) == 0xF;
                self.h += 1;
                self.f.set_z(self.h == 0);
                self.f.set_n(false);
                self.f.set_h(h);
            }

            0x34 => {
                // INC (HL)
                let addr = self.get_reg16(Reg16::HL);
                let value = bus.read_byte(addr) + 1;
                bus.write_byte(addr, value);
                let h = (value & 0xF) == 0;
                self.f.set_z(value == 0);
                self.f.set_n(false);
                self.f.set_h(h);
            }

            0x05 => {
                // DEC B
                let h = (self.h & 0xF) == 0;
                self.f.set_z(self.h == 0);
                self.f.set_n(false);
                self.f.set_h(h);
            }

            0x40..=0x7F => {
                // LD R, R
                let dst = Reg8::from_bits(op >> 3);
                let src = Reg8::from_bits(op);
                let value = self.get_reg8(bus, src);
                self.set_reg8(bus, dst, value);
            }

            0x80..=0x87 => {
                // ADD R, R
                let dst = Reg8::from_bits(op >> 3);
                let src = Reg8::from_bits(op);
                let dst_val = self.get_reg8(bus, dst);
                let src_val = self.get_reg8(bus, src);
                let (value, flags) = checked_add8(dst_val, src_val, false);
                self.f = flags;
                self.set_reg8(bus, dst, value);
            }

            0x88..=0x8F => {
                // ADC R, R
                let dst = Reg8::from_bits(op >> 3);
                let src = Reg8::from_bits(op);
                let dst_val = self.get_reg8(bus, dst);
                let src_val = self.get_reg8(bus, src);
                let carry = self.f.c();
                let (value, flags) = checked_add8(dst_val, src_val, carry);
                self.f = flags;
                self.set_reg8(bus, dst, value);
            }

            0x90..=0x97 => {
                // SUB R, R
                let dst = Reg8::from_bits(op >> 3);
                let src = Reg8::from_bits(op);
                let dst_val = self.get_reg8(bus, dst);
                let src_val = self.get_reg8(bus, src);
                let (value, flags) = checked_sub8(dst_val, src_val, false);
                self.f = flags;
                self.set_reg8(bus, dst, value);
            }

            0x98..=0x9F => {
                // SBC R, R
                let dst = Reg8::from_bits(op >> 3);
                let src = Reg8::from_bits(op);
                let dst_val = self.get_reg8(bus, dst);
                let src_val = self.get_reg8(bus, src);
                let carry = self.f.c();
                let (value, flags) = checked_sub8(dst_val, src_val, carry);
                self.f = flags;
                self.set_reg8(bus, dst, value);
            }

            0xA0..=0xA7 => {
                // AND R
                let src = Reg8::from_bits(op);
                let src_val = self.get_reg8(bus, src);
                self.a &= src_val;
                self.f.set_z(self.a == 0);
                self.f.set_n(false);
                self.f.set_c(false);
                self.f.set_h(false);
            }

            0xA8..=0xAF => {
                // XOR R
                let src = Reg8::from_bits(op);
                let src_val = self.get_reg8(bus, src);
                self.a ^= src_val;
                self.f.set_z(self.a == 0);
                self.f.set_n(false);
                self.f.set_c(false);
                self.f.set_h(false);
            }

            0xB0..=0xB7 => {
                // OR R
                let src = Reg8::from_bits(op);
                let src_val = self.get_reg8(bus, src);
                self.a |= src_val;
                self.f.set_z(self.a == 0);
                self.f.set_n(false);
                self.f.set_c(false);
                self.f.set_h(false);
            }

            0xB8..=0xBF => {
                // CP R
                let reg = Reg8::from_bits(op >> 3);
                let a_val = self.get_reg8(bus, Reg8::A);
                let reg_val = self.get_reg8(bus, reg);
                let (_, flags) = checked_sub8(a_val, reg_val, false);
                self.f = flags;
            }
            _ => panic!("Unhandled opcode {op:02X}"),
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
            }
            Reg16::BC => {
                self.b = hi;
                self.c = lo;
            }
            Reg16::DE => {
                self.d = hi;
                self.e = lo;
            }
            Reg16::HL => {
                self.h = hi;
                self.l = lo;
            }
        }
    }

    fn next8(&mut self, bus: &mut Bus) -> u8 {
        let byte = bus.read_byte(self.pc);
        self.pc += 1;
        byte
    }

    fn next16(&mut self, bus: &mut Bus) -> u16 {
        let lo = self.next8(bus);
        let hi = self.next8(bus);
        u16::from_be_bytes([hi, lo])
    }
}
