use super::comp_mode::AllowedInstructions;



#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Instruction {
    // Here begin the original Chip8 instructions
    ClearScreen,
    Return,
    Jump(Address),
    Call(Address),
    SkipEqualConstant(Register, Constant),
    SkipNotEqualConstant(Register, Constant),
    SkipEqual(Register, Register),
    Set(Register, Constant),
    SetSum(Register, Constant),
    Mov(Register, Register),
    Or(Register, Register),
    And(Register, Register),
    Xor(Register, Register),
    Add(Register, Register),
    Sub(Register, Register),
    ShiftRight(Register, Register),
    RevSub(Register, Register),
    ShiftLeft(Register, Register),
    SkipNotEqual(Register, Register),
    LoadI(Address),
    JumpRelative(Address),
    Random(Register, Constant),
    Draw(Register, Register, Constant),
    SkipPressed(Register),
    SkipNotPressed(Register),
    LoadDelay(Register),
    WaitForKey(Register),
    StoreDelay(Register),
    StoreSound(Register),
    AddI(Register),
    LoadSprite(Register),
    StoreBCD(Register),
    Store(Register),
    Load(Register),
    ScrollDown(Constant),

    // Here begin the SuperChip instructions
    ScrollRight,
    ScrollLeft,
    Exit,
    LoRes,
    HiRes,
    LoadLargeSprite(Register),
    StoreUserFlags(Register),
    LoadUserFlags(Register),

    // Here begin the XO-Chip instructions
    // todo
}
impl Instruction {
    pub fn decode(bytes: &[u8]) -> Option<Instruction> {
        let x = extract_x(bytes);
        let y = extract_y(bytes);
        let n = extract_n(bytes);
        let kk = extract_kk(bytes);
        let nnn = extract_nnn(bytes);

        let nibbles = extract_nibbles(bytes);

        Some(match nibbles {
            [0x0, 0x0, 0xC,   _] => Instruction::ScrollDown(n),
            [0x0, 0x0, 0xE, 0x0] => Instruction::ClearScreen,
            [0x0, 0x0, 0xE, 0xE] => Instruction::Return,
            [0x0, 0x0, 0xF, 0xB] => Instruction::ScrollRight,
            [0x0, 0x0, 0xF, 0xC] => Instruction::ScrollLeft,
            [0x0, 0x0, 0xF, 0xD] => Instruction::Exit,
            [0x0, 0x0, 0xF, 0xE] => Instruction::LoRes,
            [0x0, 0x0, 0xF, 0xF] => Instruction::HiRes,
            [0x1,   _,   _,   _] => Instruction::Jump(nnn),
            [0x2,   _,   _,   _] => Instruction::Call(nnn),
            [0x3,   _,   _,   _] => Instruction::SkipEqualConstant(x, kk),
            [0x4,   _,   _,   _] => Instruction::SkipNotEqualConstant(x, kk),
            [0x5,   _,   _, 0x0] => Instruction::SkipEqual(x, y),
            [0x6,   _,   _,   _] => Instruction::Set(x, kk),
            [0x7,   _,   _,   _] => Instruction::SetSum(x, kk),
            [0x8,   _,   _, 0x0] => Instruction::Mov(x, y),
            [0x8,   _,   _, 0x1] => Instruction::Or(x, y),
            [0x8,   _,   _, 0x2] => Instruction::And(x, y),
            [0x8,   _,   _, 0x3] => Instruction::Xor(x, y),
            [0x8,   _,   _, 0x4] => Instruction::Add(x, y),
            [0x8,   _,   _, 0x5] => Instruction::Sub(x, y),
            [0x8,   _,   _, 0x6] => Instruction::ShiftRight(x, y),
            [0x8,   _,   _, 0x7] => Instruction::RevSub(x, y),
            [0x8,   _,   _, 0xE] => Instruction::ShiftLeft(x, y),
            [0x9,   _,   _, 0x0] => Instruction::SkipNotEqual(x, y),
            [0xA,   _,   _,   _] => Instruction::LoadI(nnn),
            [0xB,   _,   _,   _] => Instruction::JumpRelative(nnn),
            [0xC,   _,   _,   _] => Instruction::Random(x, kk),
            [0xD,   _,   _,   _] => Instruction::Draw(x, y, n),
            [0xE,   _, 0x9, 0xE] => Instruction::SkipPressed(x),
            [0xE,   _, 0xA, 0x1] => Instruction::SkipNotPressed(x),
            [0xF,   _, 0x0, 0x7] => Instruction::LoadDelay(x),
            [0xF,   _, 0x0, 0xA] => Instruction::WaitForKey(x),
            [0xF,   _, 0x1, 0x5] => Instruction::StoreDelay(x),
            [0xF,   _, 0x1, 0x8] => Instruction::StoreSound(x),
            [0xF,   _, 0x1, 0xE] => Instruction::AddI(x),
            [0xF,   _, 0x2, 0x9] => Instruction::LoadSprite(x),
            [0xF,   _, 0x3, 0x0] => Instruction::LoadLargeSprite(x),
            [0xF,   _, 0x3, 0x3] => Instruction::StoreBCD(x),
            [0xF,   _, 0x5, 0x5] => Instruction::Store(x),
            [0xF,   _, 0x6, 0x5] => Instruction::Load(x),
            [0xF,   _, 0x7, 0x5] => Instruction::StoreUserFlags(x),
            [0xF,   _, 0x8, 0x5] => Instruction::LoadUserFlags(x),
            _ => return None,
        })
    }

    pub fn needed_comp(&self) -> AllowedInstructions {
        use Instruction::*;
        use AllowedInstructions::*;
        match self {
            ClearScreen => Original,
            Return => Original,
            Jump(_) => Original,
            Call(_) => Original,
            SkipEqualConstant(_, _) => Original,
            SkipNotEqualConstant(_, _) => Original,
            SkipEqual(_, _) => Original,
            Set(_, _) => Original,
            SetSum(_, _) => Original,
            Mov(_, _) => Original,
            Or(_, _) => Original,
            And(_, _) => Original,
            Xor(_, _) => Original,
            Add(_, _) => Original,
            Sub(_, _) => Original,
            ShiftRight(_, _) => Original,
            RevSub(_, _) => Original,
            ShiftLeft(_, _) => Original,
            SkipNotEqual(_, _) => Original,
            LoadI(_) => Original,
            JumpRelative(_) => Original,
            Random(_, _) => Original,
            Draw(_, _, _) => Original,
            SkipPressed(_) => Original,
            SkipNotPressed(_) => Original,
            LoadDelay(_) => Original,
            WaitForKey(_) => Original,
            StoreDelay(_) => Original,
            StoreSound(_) => Original,
            AddI(_) => Original,
            LoadSprite(_) => Original,
            StoreBCD(_) => Original,
            Store(_) => Original,
            Load(_) => Original,
            ScrollDown(_) => Original,

            ScrollRight => SuperChip,
            ScrollLeft => SuperChip,
            Exit => SuperChip,
            LoRes => SuperChip,
            HiRes => SuperChip,
            LoadLargeSprite(_) => SuperChip,
            StoreUserFlags(_) => SuperChip,
            LoadUserFlags(_) => SuperChip,
        }
    }

    pub fn length(&self) -> u16 {
        match self {
            _ => 2,
        }
    }
}

fn extract_x(bytes: &[u8]) -> Register {
    let x = bytes[0] & 0x0F;
    Register(x)
}
fn extract_y(bytes: &[u8]) -> Register {
    let y = (bytes[1] & 0xF0) >> 4;
    Register(y)
}
fn extract_n(bytes: &[u8]) -> Constant {
    Constant(bytes[1] & 0x0F)
}
fn extract_kk(bytes: &[u8]) -> Constant {
    Constant(bytes[1])
}
fn extract_nnn(bytes: &[u8]) -> Address {
    let low = bytes[1] as u16;
    let high = (bytes[0] & 0x0F) as u16;
    let addr = (high << 8) | low;
    Address(addr)
}
fn extract_nibbles(bytes: &[u8]) -> [u8; 4] {
    let highest = (bytes[0] & 0xF0) >> 4;
    let mid_high = bytes[0] & 0x0F;
    let mid_low = (bytes[1] & 0xF0) >> 4;
    let lowest = bytes[1] & 0x0F;

    [highest, mid_high, mid_low, lowest]
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Register(pub u8);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Constant(pub u8);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Address(pub u16);
