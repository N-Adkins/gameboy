#[derive(Debug)]
pub struct Bus {
    rom: Vec<u8>,
    vram: [u8; 0x2000],
    wram: [u8; 0x2000],
}

impl Bus {
    pub fn read_byte(&mut self, _addr: u16) -> u8 {
        todo!()
    }

    pub fn write_byte(&mut self, _addr: u16, _value: u8) {
        todo!()
    }
}
