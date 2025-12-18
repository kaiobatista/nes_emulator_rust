pub mod Button {
    pub const A: u8 = 1 << 7;
    pub const B: u8 = 1 << 6;
    pub const SELECT: u8 = 1 << 5;
    pub const START: u8 = 1 << 4;
    pub const UP: u8 = 1 << 3;
    pub const DOWN: u8 = 1 << 2; 
    pub const LEFT: u8 = 1 << 1;
    pub const RIGHT:u8 = 1 << 0;
}

pub struct Controller {
    pub buttons: u8,

    index: u8,
    strobe: u8,
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            buttons: 0,
            index: 0,
            strobe: 0,
        }
    }

    pub fn write(&mut self, data: u8) {
        self.strobe = data;
        if self.strobe & 1 == 1 {
            self.index = 0;
        }
    }

    pub fn read(&mut self) -> u8 {
        let mut value = 0;

        if self.strobe & 1 == 1 {
            value = (self.buttons >> 7) & 1;
        } else {

            if self.index < 8 {
                value = (self.buttons >> (7 - self.index)) & 1;
            } else {
                value = 1
            }

            self.index += 1;
        }

        value | 0x40
    }

    pub fn set_button(&mut self, button_mask: u8, pressed: bool) {
        if pressed {
            self.buttons |= button_mask;
        } else {
            self.buttons &= !button_mask;
        }
    }
}
