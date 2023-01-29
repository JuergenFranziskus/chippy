

pub struct Keys {
    key_values: [bool; 16],
}
impl Keys {
    pub fn new() -> Self {
        Self {
            key_values: [false; 16],
        }
    }
    pub fn is_pressed(&self, k: u8) -> bool {
        assert!(k < 16);
        self.key_values[k as usize]
    }
    pub fn set_key(&mut self, k: u8, pressed: bool) {
        assert!(k < 16);
        self.key_values[k as usize] = pressed;
    }
}
