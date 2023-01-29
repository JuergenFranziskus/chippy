use super::instruction::Instruction;


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CompatibilityMode {
    pub shift: ShiftMode,
    pub load_store: LoadStoreMode,
    pub address_space: AddressSpace,
    pub allowed_instructions: AllowedInstructions,
    pub jump_mode: RelativeJumpMode,
    pub collisions: CollisionEnumeration,
}


pub struct CompBuilder {
    comp: CompatibilityMode
}
impl CompBuilder {
    pub fn new() -> Self {
        Self {
            comp: CompatibilityMode{
                shift: ShiftMode::SuperChip,
                load_store: LoadStoreMode::SuperChip,
                address_space: AddressSpace::Original,
                allowed_instructions: AllowedInstructions::Original,
                jump_mode: RelativeJumpMode::Original,
                collisions: CollisionEnumeration::Original,
            }
        }
    }

    pub fn superchip_preset() -> Self {
        Self {
            comp: CompatibilityMode {
                shift: ShiftMode::SuperChip,
                load_store: LoadStoreMode::SuperChip,
                address_space: AddressSpace::Original,
                allowed_instructions: AllowedInstructions::SuperChip,
                jump_mode: RelativeJumpMode::SuperChip,
                collisions: CollisionEnumeration::SuperChip,
            },
        }
    }

    pub fn with_jump_mode(mut self, mode: RelativeJumpMode) -> Self {
        self.comp.jump_mode = mode;
        self
    }

    pub fn build(self) -> CompatibilityMode {
        self.comp
    }
}



#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ShiftMode {
    /// Shift Vy into Vx, leaving Vy unchanged
    Original,
    /// Shift Vx into itself, completely ignoring Vy
    SuperChip,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum LoadStoreMode {
    /// Leave I incremented after load/store instruction
    Original,
    /// Leave I unchaged after load/store instruction
    SuperChip,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AddressSpace {
    /// Treat I as 12-bit pointer, modulo 4096
    Original,
    /// Treat I as 16-bit pointer, modulo 65536
    XOChip,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AllowedInstructions {
    /// Allow only instructions found on the original Chip8
    Original = 0,
    /// Allow only instructions found on SuperChip
    SuperChip = 1,
    /// Allow all instructions, including ones unique to XOChip
    XOChip = 2,
}
impl AllowedInstructions {
    pub fn is_legal(self, instruction: &Instruction) -> bool {
        let needs = instruction.needed_comp() as u8;
        let is = self as u8;

        is >= needs
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum RelativeJumpMode {
    /// Always use V0 as the base register
    Original,
    /// Treat the highest byte of the 3-byte address as an index to select the base register
    SuperChip,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CollisionEnumeration {
    /// Set VF equal to one if collision occured, otherwise 0
    Original,
    /// Set VF equal to the amount of collisions that occured
    SuperChip,
}
