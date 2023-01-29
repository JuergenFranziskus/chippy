use std::{io::{Write, self, stderr}, ops::{Index, IndexMut}};
use rand::prelude::*;
use super::{screen::Screen, instruction::{Instruction, Address, Register, Constant}, comp_mode::{CompatibilityMode, ShiftMode, LoadStoreMode, AddressSpace, RelativeJumpMode}, keys::Keys};

const MEMORY_SIZE: usize = 2usize.pow(16);

pub struct Machine {
    cpu: CPU,
    stack: Vec<u16>,
    memory: Box<[u8; MEMORY_SIZE]>,
    screen: Screen,
    rng: StdRng,
}
impl Machine {
    pub fn new(rng_seed: u64) -> Machine {
        Machine {
            cpu: CPU::new(),
            stack: Vec::new(),
            memory: Box::new([0; MEMORY_SIZE]),
            screen: Screen::new(),
            rng: StdRng::seed_from_u64(rng_seed),
        }
    }

    pub fn decode_and_execute(&mut self, comp: &CompatibilityMode, keys: &Keys) {
        let instruction = self.decode();
        self.assert_legal(&instruction, comp);

        self.cpu.ip += instruction.length();

        let skip = self.cpu.skip;
        self.cpu.skip = false;

        if !skip {
            self.execute(instruction, comp, keys);
        }
    }
    fn decode(&self) -> Instruction {
        let instruction = Instruction::decode(&self.memory[self.cpu.ip as usize..]);

        let Some(instruction) = instruction else {
            let ip = self.cpu.ip as usize;
            panic!("Invalid instruction at {:x?}", &self.memory[ip..ip+4]);
        };

        instruction
    }
    fn assert_legal(&self, i: &Instruction, comp: &CompatibilityMode) {
        let allowed = comp.allowed_instructions;
        if !allowed.is_legal(i) {
            let ip = self.cpu.ip as usize;
            panic!("Instruction {:?} at address {:x} is not legal in compatibility mode {:?}", i, ip, allowed);
        }
    }
    fn execute(&mut self, i: Instruction, comp: &CompatibilityMode, keys: &Keys) {
        use Instruction::*;
        match i {
            ClearScreen => self.exec_clear_screen(),
            Return => self.exec_return(),
            HiRes => self.exec_hires(),
            Jump(nnn) => self.exec_jump(nnn),
            Call(nnn) => self.exec_call(nnn),
            SkipEqualConstant(x, kk) => self.exec_skip_equal_constant(x, kk),
            SkipNotEqualConstant(x, kk) => self.exec_skip_not_equal_constant(x, kk),
            SkipEqual(x, kk) => self.exec_skip_equal(x, kk),
            Set(x, kk) => self.exec_set(x, kk),
            SetSum(x, kk) => self.exec_set_sum(x, kk),
            Mov(x, y) => self.exec_mov(x, y),
            Or(x, y) => self.exec_or(x, y),
            And(x, y) => self.exec_and(x, y),
            Xor(x, y) => self.exec_xor(x, y),
            Add(x, y) => self.exec_add(x, y),
            Sub(x, y) => self.exec_sub(x, y),
            ShiftRight(x, y) => self.exec_shift_right(x, y, comp),
            RevSub(x, y) => self.exec_rev_sub(x, y),
            ShiftLeft(x, y) => self.exec_shift_left(x, y, comp),
            SkipNotEqual(x, y) => self.exec_skip_not_equal(x, y),
            LoadI(nnn) => self.exec_load_i(nnn),
            JumpRelative(nnn) => self.exec_jump_relative(nnn, comp),
            Random(x, kk) => self.exec_random(x, kk),
            Draw(x, y, n) => self.exec_draw(x, y, n),
            SkipNotPressed(x) => self.exec_skip_not_pressed(x, keys),
            LoadDelay(x) => self.exec_load_delay(x),
            WaitForKey(x) => self.exec_wait_for_key(x, keys),
            StoreSound(x) => self.exec_store_sound(x),
            StoreDelay(x) => self.exec_store_delay(x),
            AddI(x) => self.exec_add_i(x, comp),
            LoadSprite(x) => self.exec_load_sprite(x),
            LoadLargeSprite(x) => self.exec_load_hires_sprite(x),
            StoreBCD(x) => self.exec_store_bcd(x),
            Store(x) => self.exec_store(x, comp),
            Load(x) => self.exec_load(x, comp),
            StoreUserFlags(x) => self.exec_store_user_flags(x),
            LoadUserFlags(x) => self.exec_load_user_flags(x),

            _ => {
                self.screen.write(stderr()).unwrap();
                panic!("Unimplemented instruction {:x?} at address {:x}", i, self.cpu.ip - i.length());
            },
        }
    }

    fn exec_clear_screen(&mut self) {
        self.screen.clear();
    }
    fn exec_return(&mut self) {
        let ip = self.stack.pop().unwrap();
        self.cpu.ip = ip;
    }
    fn exec_hires(&mut self) {
        self.screen.enable_hires();
    }
    fn exec_jump(&mut self, nnn: Address) {
        self.cpu.ip = nnn.0;
    }
    fn exec_call(&mut self, nnn: Address) {
        self.stack.push(self.cpu.ip);
        self.cpu.ip = nnn.0;
    }
    fn exec_skip_equal_constant(&mut self, x: Register, kk: Constant) {
        if self.cpu[x] == kk.0 {
            self.cpu.skip = true;
        }
    }
    fn exec_skip_not_equal_constant(&mut self, x: Register, kk: Constant) {
        if self.cpu[x] != kk.0 {
            self.cpu.skip = true;
        }
    }
    fn exec_skip_equal(&mut self, x: Register, y: Register) {
        if self.cpu[x] == self.cpu[y] {
            self.cpu.skip = true;
        }
    }
    fn exec_set(&mut self, x: Register, kk: Constant) {
        self.cpu[x] = kk.0;
    }
    fn exec_set_sum(&mut self, x: Register, kk: Constant) {
        let sum = self.cpu[x].wrapping_add(kk.0);
        self.cpu[x] = sum;
    }
    fn exec_mov(&mut self, x: Register, y: Register) {
        self.cpu[x] = self.cpu[y];
    }
    fn exec_or(&mut self, x: Register, y: Register) {
        self.cpu[x] |= self.cpu[y];
    }
    fn exec_and(&mut self, x: Register, y: Register) {
        self.cpu[x] &= self.cpu[y];
    }
    fn exec_xor(&mut self, x: Register, y: Register) {
        self.cpu[x] ^= self.cpu[y];
    }
    fn exec_add(&mut self, x: Register, y: Register) {
        let (sum, carry) = self.cpu[x].overflowing_add(self.cpu[y]);
        self.cpu[x] = sum;
        self.cpu.registers[0xF] = if carry { 1 } else { 0 };
    }
    fn exec_sub(&mut self, x: Register, y: Register) {
        let (diff, borrow) = self.cpu[x].overflowing_sub(self.cpu[y]);
        self.cpu[x] = diff;
        self.cpu.registers[0xF] = if !borrow { 1 } else { 0 };
    }
    fn exec_shift_right(&mut self, x: Register, y: Register, comp: &CompatibilityMode) {
        let shift = if comp.shift == ShiftMode::Original { y } else { x };
        let (shifted, carry) = self.cpu[shift].overflowing_shr(1);
        self.cpu[x] = shifted;
        self.cpu.registers[0xF] = if carry { 1 } else { 0 };
    }
    fn exec_rev_sub(&mut self, x: Register, y: Register) {
        let (diff, borrow) = self.cpu[y].overflowing_sub(self.cpu[x]);
        self.cpu[x] = diff;
        self.cpu.registers[0xF] = if !borrow { 1 } else { 0 };
    }
    fn exec_shift_left(&mut self, x: Register, y: Register, comp: &CompatibilityMode) {
        let shift = if comp.shift == ShiftMode::Original { y } else { x };
        let (shifted, carry) = self.cpu[shift].overflowing_shl(1);
        self.cpu[x] = shifted;
        self.cpu.registers[0xF] = if carry { 1 } else { 0 };
    }
    fn exec_skip_not_equal(&mut self, x: Register, y: Register) {
        if self.cpu[x] != self.cpu[y] {
            self.cpu.skip = true;
        }
    }
    fn exec_load_i(&mut self, nnn: Address) {
        self.cpu.i = nnn.0;
    }
    fn exec_jump_relative(&mut self, nnn: Address, comp: &CompatibilityMode) {
        let nnn = nnn.0;
        let x = match comp.jump_mode {
            RelativeJumpMode::Original => 0,
            RelativeJumpMode::SuperChip => (nnn & 0xF00) >> 8,
        };

        let x = self.cpu.registers[x as usize] as u16;
        self.cpu.ip = nnn + x;
    }
    fn exec_random(&mut self, x: Register, kk: Constant) {
        let kk = kk.0;
        let value = self.rng.gen::<u8>() & kk;
        self.cpu[x] = value;
    }
    fn exec_draw(&mut self, x: Register, y: Register, n: Constant) {
        let x = self.cpu[x] as usize;
        let y = self.cpu[y] as usize;
        let i = self.cpu.i as usize;
        let sprite = &self.memory[i..];

        let collisions = self.screen.draw_sprite(sprite, x, y, n.0 as usize);

        if self.screen.is_lowres() && collisions != 0 {
            self.cpu.registers[0xF] = 1;
        }
        else if !self.screen.is_lowres() {
            self.cpu.registers[0xF] = collisions.max(255) as u8;
        }
    }
    fn exec_skip_not_pressed(&mut self, x: Register, keys: &Keys) {
        let x = self.cpu[x];
        if !keys.is_pressed(x) {
            self.cpu.skip = true;
        }
    }
    fn exec_load_delay(&mut self, x: Register) {
        self.cpu[x] = self.cpu.delay_timer;
    }
    fn exec_wait_for_key(&mut self, x: Register, keys: &Keys) {
        for k in 0..16 {
            if keys.is_pressed(k) {
                self.cpu[x] = k;
                return;
            }
        }
        self.cpu.ip -= 2;
    }
    fn exec_store_sound(&mut self, x: Register) {
        self.cpu.sound_timer = self.cpu[x];
    }
    fn exec_store_delay(&mut self, x: Register) {
        self.cpu.delay_timer = self.cpu[x];
    }
    fn exec_add_i(&mut self, x: Register, comp: &CompatibilityMode) {
        let x = self.cpu[x];
        self.cpu.i = self.cpu.i.wrapping_add(x as u16);

        if comp.address_space == AddressSpace::Original {
            self.cpu.i %= 4096;
        }
    }
    fn exec_load_sprite(&mut self, x: Register) {
        let x = self.cpu[x] as u16;
        let addr = Self::lores_sprite_start() + x * 5;
        self.cpu.i = addr;
    }
    fn exec_load_hires_sprite(&mut self, x: Register) {
        let x = self.cpu[x] as u16;
        let addr = Self::hires_sprite_start() + x * 10;
        self.cpu.i = addr;
    }
    fn exec_store_bcd(&mut self, x: Register) {
        let x = self.cpu[x];
        let ones = x % 10;
        let tens = (x / 10) % 10;
        let hundreds = (x / 10 / 10) % 10;

        let i = self.cpu.i as usize;
        self.memory[i + 0] = hundreds;
        self.memory[i + 1] = tens;
        self.memory[i + 2] = ones;
    }
    fn exec_store(&mut self, x: Register, comp: &CompatibilityMode) {
        let x = x.0 as usize;
        let i = self.cpu.i as usize;
        let regs = &self.cpu.registers[..=x];
        let mem = &mut self.memory[i..=i+x];
        mem.copy_from_slice(regs);

        if comp.load_store == LoadStoreMode::Original {
            self.cpu.i += x as u16;
        }
    }
    fn exec_load(&mut self, x: Register, comp: &CompatibilityMode) {
        let x = x.0 as usize;
        let i = self.cpu.i as usize;
        let regs = &mut self.cpu.registers[..=x];
        let mem = &self.memory[i..=i+x];
        regs.copy_from_slice(mem);

        if comp.load_store == LoadStoreMode::Original {
            self.cpu.i += x as u16;
        }
    }
    fn exec_store_user_flags(&mut self, _x: Register) {

    }
    fn exec_load_user_flags(&mut self, _x: Register) {

    }

    pub fn decrement_counters(&mut self) {
        if self.cpu.sound_timer != 0 {
            self.cpu.sound_timer -= 1;
        }
        if self.cpu.delay_timer != 0 {
            self.cpu.delay_timer -= 1;
        }
    }

    pub fn init_instruction_pointer(&mut self, ip: u16) {
        self.cpu.ip = ip;
    }
    pub fn load_program(&mut self, program: &[u8], start: usize) {
        let size = program.len();
        let dest = &mut self.memory[start..start+size];
        let src = program;
        dest.copy_from_slice(src);
    }
    pub fn load_sprites(&mut self) {
        self.load_lowres_sprites();
        self.load_hires_sprites();
    }
    fn load_lowres_sprites(&mut self) {
        for (i, &b) in SPRITE_BYTES.iter().enumerate() {
            self.memory[i] = b;
        }
    }
    fn load_hires_sprites(&mut self) {
        for (i, &b) in SPRITE_BYTES.iter().enumerate() {
            self.memory[i * 2 + 0] = b;
            self.memory[i * 2 + 1] = b;
        }
    }
    fn lores_sprite_start() -> u16 {
        0
    }
    fn hires_sprite_start() -> u16 {
        Self::lores_sprite_start() + 5 * 16
    }

    pub fn screen(&self) -> &Screen {
        &self.screen
    }
    pub fn write_screen<O: Write>(&self, out: O) -> io::Result<()> {
        self.screen.write(out)
    }
}


pub struct CPU {
    registers: [u8; 16],
    i: u16,
    ip: u16,
    skip: bool,
    sound_timer: u8,
    delay_timer: u8,
}
impl CPU {
    pub fn new() -> CPU {
        CPU {
            registers: [0; 16],
            i: 0,
            ip: 0x200,
            skip: false,
            sound_timer: 0,
            delay_timer: 0,
        }
    }
}
impl Index<Register> for CPU {
    type Output = u8;
    
    fn index(&self, index: Register) -> &Self::Output {
        &self.registers[index.0 as usize]
    }
}
impl IndexMut<Register> for CPU {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        &mut self.registers[index.0 as usize]
    }
}


static SPRITE_BYTES: &[u8] = &[
    0xF0,
    0x90,
    0x90,
    0x90,
    0xF0,
    0x20,
    0x60,
    0x20,
    0x20,
    0x70,
    0xF0,
    0x10,
    0xF0,
    0x80,
    0xF0,
    0xF0,
    0x10,
    0xF0,
    0x10,
    0xF0,
    0x90,
    0x90,
    0xF0,
    0x10,
    0x10,
    0xF0,
    0x80,
    0xF0,
    0x10,
    0xF0,
    0xF0,
    0x80,
    0xF0,
    0x90,
    0xF0,
    0xF0,
    0x10,
    0x20,
    0x40,
    0x40,
    0xF0,
    0x90,
    0xF0,
    0x90,
    0xF0,
    0xF0,
    0x90,
    0xF0,
    0x10,
    0xF0,
    0xF0,
    0x90,
    0xF0,
    0x90,
    0x90,
    0xE0,
    0x90,
    0xE0,
    0x90,
    0xE0,
    0xF0,
    0x80,
    0x80,
    0x80,
    0xF0,
    0xE0,
    0x90,
    0x90,
    0x90,
    0xE0,
    0xF0,
    0x80,
    0xF0,
    0x80,
    0xF0,
    0xF0,
    0x80,
    0xF0,
    0x80,
    0x80,
];

