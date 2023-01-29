use std::io::{Write, self};

const PLANES: usize = 2;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Screen {
    planes: [BitPlane; PLANES],
    plane_selected: [bool; PLANES],
    mode: ScreenMode,
}
impl Screen {
    pub fn new() -> Self {
        Self {
            planes: [BitPlane::new(); 2],
            plane_selected: [true, false],
            mode: ScreenMode::LowRes,
        }
    }

    pub fn disable_hires(&mut self) {
        self.mode = ScreenMode::LowRes;
    }
    pub fn enable_hires(&mut self) {
        self.mode = ScreenMode::HighRes
    }

    pub fn clear(&mut self) {
        for (plane, sel) in self.planes.iter_mut().zip(self.plane_selected) {
            if sel {
                plane.clear();
            }
        }
    }

    pub fn is_lowres(&self) -> bool {
        self.mode == ScreenMode::LowRes
    }

    pub fn write<O: Write>(&self, mut out: O) -> io::Result<()> {
        for row in 0..64 {
            for column in 0..128 {
                let bit0 = self.planes[0].rows[row] & (1 << (127 - column));
                let bit1 = self.planes[1].rows[row] & (1 << (127 - column));

                let bit0 = bit0 != 0;
                let bit1 = bit1 != 0;
                let c = match (bit0, bit1) {
                    (false, false) => ' ',
                    (true, false) => 'O',
                    (false, true) => '+',
                    (true, true) => '@',
                };

                write!(out, "{}", c)?;
            }
            writeln!(out)?;
        }

        Ok(())
    }
    pub fn render_to_pixel_buffer(&self, buffer: &mut [u8]) {
        for (i, pixel) in buffer.chunks_exact_mut(4).enumerate() {
            let y = i / WIDTH;
            let x = i % WIDTH;
            let value = self.get_pixel(x, y);
            let color = match value {
                0 => [0, 0, 0],
                1 => [255, 255, 255],
                2 => [0, 255, 0],
                3 => [128, 240, 128],
                _ => unreachable!(),
            };

            pixel[0] = color[0];
            pixel[1] = color[1];
            pixel[2] = color[2];
            pixel[3] = 0xFF;
        }
    }
    fn get_pixel(&self, x: usize, y: usize) -> u8 {
        let mut value = 0;
        for (i, plane) in self.planes.iter().enumerate() {
            let row = plane.rows[y];
            let column_mask = 1 << (WIDTH - 1 - x);
            let bit = row & column_mask != 0;
            if bit {
                value |= 1 << i;
            }
        }

        value
    }

    pub fn draw_sprite(&mut self, sprite: &[u8], x: usize, y: usize, height: usize) -> usize {
        let mut collisions = 0;

        let sprite_size = if height == 0 {
            if self.is_lowres() {
                16
            }
            else {
                32
            }
        }
        else {
            height
        };

        let mut offset = 0;
        for i in 0..PLANES {
            if self.plane_selected[i] {
                let start = offset * sprite_size;
                offset += 1;
                collisions += self.draw_to_plane(i, &sprite[start..], x, y, height);
            }
        }

        collisions
    }
    fn draw_to_plane(&mut self, plane: usize, sprite: &[u8], x: usize, y: usize, height: usize) -> usize {
        let mut collisions = 0;

        let bytes_per_row = if height == 0 {
            if self.is_lowres() {
                1
            }
            else {
                2
            }
        }
        else {
            1
        };
        let height = if height == 0 { 16 } else { height };


        for (row, sprite_bytes) in sprite.chunks_exact(bytes_per_row).take(height).enumerate() {
            let y = y + row;

            for (column_offset, &sprite_byte) in sprite_bytes.into_iter().enumerate() {
                for column in 0..8 {
                    let x = x + column + column_offset * 8;
                    let mask = 1 << (7 - column);
                    let bit = (sprite_byte & mask) != 0;
                    if self.planes[plane].draw_pixel(x, y, bit, self.is_lowres()) {
                        collisions += 1;
                    }
                }
            }
        }

        collisions
    }
}

const WIDTH: usize = 128;
const HEIGHT: usize = 64;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BitPlane {
    rows: [u128; HEIGHT],
}
impl BitPlane {
    pub fn new() -> Self {
        Self {
            rows: [0; HEIGHT],
        }
    }

    pub fn clear(&mut self) {
        self.rows = [0; HEIGHT];
    }

    fn draw_pixel(&mut self, mut x: usize, mut y: usize, pixel: bool, lores: bool) -> bool {
        if lores {
            x *= 2;
            y *= 2;
        }
        
        let mut collision = false;
        let pixel_mask = if pixel { u128::MAX } else { 0 };

        let limit = if lores { 2 } else { 1 };
        for y_off in 0..limit {
            let y = y + y_off;
            
            for x_off in 0..limit {
                let x = x + x_off;

                if x >= WIDTH || y >= HEIGHT {
                    if !lores {
                        collision = true;
                        break;
                    }
                }
                let x = x % WIDTH;
                let y = y % HEIGHT;

                let row = &mut self.rows[y];
                let mask = pixel_mask & 1 << (WIDTH - 1 - x);
                if *row & mask != 0 {
                    collision = true;
                }
                *row ^= mask;
            }
        }

        collision
    }
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ScreenMode {
    HighRes,
    LowRes,
}
