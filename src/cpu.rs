use super::bus::Bus;
use super::bus::Mem;
use std::collections::HashMap;
mod op;
pub mod op_test;
mod trace_test;

#[derive(Clone)]
pub struct OpCode {
    pub name: String,
    pub op_length: u8,
    pub cycles: u8,
    pub mode: AddressingMode,
}

//#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct CPU<'call> {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    pub stack_counter: u8,
    pub bus: Bus<'call>,

    pub op_map: HashMap<u8, OpCode>,

    // crucial details about added cpu's cycles
    pub added_cycles_of_addr: u8,
    pub added_cycles_of_br: u8,
}

#[derive(Debug, Clone, PartialEq, Copy)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect, // TODO: strange addr mode
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}

#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
#[repr(u8)]
enum StatusFlag {
    Carry = 0b0000_0001,
    Zero = 0b0000_0010,
    InterruptDisable = 0b0000_0100,
    DecimalMode = 0b0000_1000,
    BreakCommand = 0b0001_0000,
    Undefined = 0b0010_0000,
    Overflow = 0b0100_0000,
    Negative = 0b1000_0000,
}

impl StatusFlag {
    fn reverse(&self) -> u8 {
        return 0b1111_1111 ^ (*self as u8);
    }

    fn among(&self, status: u8) -> bool {
        return status & (*self as u8) == (*self as u8);
    }

    fn add(&self, status: &mut u8) {
        *status = *status | *self;
    }
    fn remove(&self, status: &mut u8) {
        *status = *status & !(*self as u8);
    }
    fn test(&self, status: u8) -> bool {
        return *self & status == *self;
    }
    fn or_get(&self, status: u8) -> u8 {
        return status | *self;
    }
    fn and_get(&self, status: u8) -> u8 {
        return status & *self;
    }
}

use std::ops::BitAnd;
impl BitAnd<u8> for StatusFlag {
    type Output = u8;
    fn bitand(self, rhs: u8) -> Self::Output {
        (self as u8) & rhs
    }
}
impl BitAnd<StatusFlag> for u8 {
    type Output = u8;
    fn bitand(self, rhs: StatusFlag) -> Self::Output {
        self & (rhs as u8)
    }
}

use std::ops::BitOr;
impl BitOr<u8> for StatusFlag {
    type Output = u8;
    fn bitor(self, rhs: u8) -> Self::Output {
        (self as u8) | rhs
    }
}
impl BitOr<StatusFlag> for u8 {
    type Output = u8;
    fn bitor(self, rhs: StatusFlag) -> Self::Output {
        self | (rhs as u8)
    }
}

impl PartialEq<u8> for StatusFlag {
    fn eq(&self, other: &u8) -> bool {
        return *other == *self as u8;
    }
}
impl PartialEq<StatusFlag> for u8 {
    fn eq(&self, other: &StatusFlag) -> bool {
        return *self == *other as u8;
    }
}

impl OpCode {
    pub fn new(name: &str, op_length: u8, cycles: u8, mode: AddressingMode) -> Self {
        OpCode {
            name: name.to_string(),
            op_length,
            cycles,
            mode,
        }
    }
}

impl<'call> Mem for CPU<'call> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data);
    }
}

impl<'call> CPU<'call> {
    pub fn new(bus: Bus<'call>) -> Self {
        let mut op_map: HashMap<u8, OpCode> = HashMap::new();

        // LDA:
        op_map.insert(0xa9, OpCode::new("LDA", 2, 2, AddressingMode::Immediate));
        op_map.insert(0xa5, OpCode::new("LDA", 2, 3, AddressingMode::ZeroPage));
        op_map.insert(0xb5, OpCode::new("LDA", 2, 4, AddressingMode::ZeroPage_X));
        op_map.insert(0xad, OpCode::new("LDA", 3, 4, AddressingMode::Absolute));
        op_map.insert(
            0xbd,
            OpCode::new(
                "LDA",
                3,
                4, /* +1 if page crossed */
                AddressingMode::Absolute_X,
            ),
        );
        op_map.insert(
            0xb9,
            OpCode::new(
                "LDA",
                3,
                4, /* +1 if page crossed */
                AddressingMode::Absolute_Y,
            ),
        );
        op_map.insert(0xa1, OpCode::new("LDA", 2, 6, AddressingMode::Indirect_X));
        op_map.insert(
            0xb1,
            OpCode::new(
                "LDA",
                2,
                5, /* +1 if page corssed */
                AddressingMode::Indirect_Y,
            ),
        );

        // LDX:
        op_map.insert(0xa2, OpCode::new("LDX", 2, 2, AddressingMode::Immediate));
        op_map.insert(0xa6, OpCode::new("LDX", 2, 3, AddressingMode::ZeroPage));
        op_map.insert(0xb6, OpCode::new("LDX", 2, 4, AddressingMode::ZeroPage_Y));
        op_map.insert(0xae, OpCode::new("LDX", 3, 4, AddressingMode::Absolute));
        op_map.insert(
            0xbe,
            OpCode::new(
                "LDX",
                3,
                4, /* +1 if page crossed */
                AddressingMode::Absolute_Y,
            ),
        );

        // LDY:
        op_map.insert(0xa0, OpCode::new("LDY", 2, 2, AddressingMode::Immediate));
        op_map.insert(0xa4, OpCode::new("LDY", 2, 3, AddressingMode::ZeroPage));
        op_map.insert(0xb4, OpCode::new("LDY", 2, 4, AddressingMode::ZeroPage_X));
        op_map.insert(0xac, OpCode::new("LDY", 3, 4, AddressingMode::Absolute));
        op_map.insert(
            0xbc,
            OpCode::new(
                "LDY",
                3,
                4, /* +1 if page crossed */
                AddressingMode::Absolute_X,
            ),
        );

        // STA:
        op_map.insert(0x85, OpCode::new("STA", 2, 3, AddressingMode::ZeroPage));
        op_map.insert(0x95, OpCode::new("STA", 2, 4, AddressingMode::ZeroPage_X));
        op_map.insert(0x8d, OpCode::new("STA", 3, 4, AddressingMode::Absolute));
        op_map.insert(
            0x9d,
            OpCode::new(
                "STA",
                3,
                4, /* +1 if page crossed */
                AddressingMode::Absolute_X,
            ),
        );
        op_map.insert(
            0x99,
            OpCode::new(
                "STA",
                3,
                4, /* +1 if page crossed */
                AddressingMode::Absolute_Y,
            ),
        );
        op_map.insert(0x81, OpCode::new("STA", 2, 6, AddressingMode::Indirect_X));
        op_map.insert(
            0x91,
            OpCode::new(
                "STA",
                2,
                5, /* +1 if page corssed */
                AddressingMode::Indirect_Y,
            ),
        );

        // STX:
        op_map.insert(0x86, OpCode::new("STX", 2, 3, AddressingMode::ZeroPage));
        op_map.insert(0x96, OpCode::new("STX", 2, 4, AddressingMode::ZeroPage_Y));
        op_map.insert(0x8e, OpCode::new("STX", 3, 4, AddressingMode::Absolute));

        // STY:
        op_map.insert(0x84, OpCode::new("STY", 2, 3, AddressingMode::ZeroPage));
        op_map.insert(0x94, OpCode::new("STY", 2, 4, AddressingMode::ZeroPage_X));
        op_map.insert(0x8c, OpCode::new("STY", 3, 4, AddressingMode::Absolute));

        // ADC:
        op_map.insert(0x69, OpCode::new("ADC", 2, 2, AddressingMode::Immediate));
        op_map.insert(0x65, OpCode::new("ADC", 2, 3, AddressingMode::ZeroPage));
        op_map.insert(0x75, OpCode::new("ADC", 2, 4, AddressingMode::ZeroPage_X));
        op_map.insert(0x6d, OpCode::new("ADC", 3, 4, AddressingMode::Absolute));
        op_map.insert(
            0x7d,
            OpCode::new(
                "ADC",
                3,
                4, /* +1 if page crossed */
                AddressingMode::Absolute_X,
            ),
        );
        op_map.insert(
            0x79,
            OpCode::new(
                "ADC",
                3,
                4, /* +1 if page crossed */
                AddressingMode::Absolute_Y,
            ),
        );
        op_map.insert(0x61, OpCode::new("ADC", 2, 6, AddressingMode::Indirect_X));
        op_map.insert(
            0x71,
            OpCode::new(
                "ADC",
                2,
                5, /* +1 if page corssed */
                AddressingMode::Indirect_Y,
            ),
        );

        // SBC:
        op_map.insert(0xe9, OpCode::new("SBC", 2, 2, AddressingMode::Immediate));
        op_map.insert(0xe5, OpCode::new("SBC", 2, 3, AddressingMode::ZeroPage));
        op_map.insert(0xf5, OpCode::new("SBC", 2, 4, AddressingMode::ZeroPage_X));
        op_map.insert(0xed, OpCode::new("SBC", 3, 4, AddressingMode::Absolute));
        op_map.insert(
            0xfd,
            OpCode::new(
                "SBC",
                3,
                4, /* +1 if page crossed */
                AddressingMode::Absolute_X,
            ),
        );
        op_map.insert(
            0xf9,
            OpCode::new(
                "SBC",
                3,
                4, /* +1 if page crossed */
                AddressingMode::Absolute_Y,
            ),
        );
        op_map.insert(0xe1, OpCode::new("SBC", 2, 6, AddressingMode::Indirect_X));
        op_map.insert(
            0xf1,
            OpCode::new(
                "SBC",
                2,
                5, /* +1 if page corssed */
                AddressingMode::Indirect_Y,
            ),
        );

        // AND: A logical AND is performed, bit by bit, on the accumulator contents using the contents of a byte of memory.
        op_map.insert(0x29, OpCode::new("AND", 2, 2, AddressingMode::Immediate));
        op_map.insert(0x25, OpCode::new("AND", 2, 3, AddressingMode::ZeroPage));
        op_map.insert(0x35, OpCode::new("AND", 2, 4, AddressingMode::ZeroPage_X));
        op_map.insert(0x2d, OpCode::new("AND", 3, 4, AddressingMode::Absolute));
        op_map.insert(
            0x3d,
            OpCode::new(
                "AND",
                3,
                4, /* +1 if page crossed */
                AddressingMode::Absolute_X,
            ),
        );
        op_map.insert(
            0x39,
            OpCode::new(
                "AND",
                3,
                4, /* +1 if page crossed */
                AddressingMode::Absolute_Y,
            ),
        );
        op_map.insert(0x21, OpCode::new("AND", 2, 6, AddressingMode::Indirect_X));
        op_map.insert(
            0x31,
            OpCode::new(
                "AND",
                2,
                5, /* +1 if page corssed */
                AddressingMode::Indirect_Y,
            ),
        );

        // ORA: An inclusive OR is performed, bit by bit, on the accumulator contents using the contents of a byte of memory.
        op_map.insert(0x09, OpCode::new("ORA", 2, 2, AddressingMode::Immediate));
        op_map.insert(0x05, OpCode::new("ORA", 2, 3, AddressingMode::ZeroPage));
        op_map.insert(0x15, OpCode::new("ORA", 2, 4, AddressingMode::ZeroPage_X));
        op_map.insert(0x0d, OpCode::new("ORA", 3, 4, AddressingMode::Absolute));
        op_map.insert(
            0x1d,
            OpCode::new(
                "ORA",
                3,
                4, /* +1 if page crossed */
                AddressingMode::Absolute_X,
            ),
        );
        op_map.insert(
            0x19,
            OpCode::new(
                "ORA",
                3,
                4, /* +1 if page crossed */
                AddressingMode::Absolute_Y,
            ),
        );
        op_map.insert(0x01, OpCode::new("ORA", 2, 6, AddressingMode::Indirect_X));
        op_map.insert(
            0x11,
            OpCode::new(
                "ORA",
                2,
                5, /* +1 if page corssed */
                AddressingMode::Indirect_Y,
            ),
        );

        // EOR(XOR): An exclusive OR is performed, bit by bit, on the accumulator contents using the contents of a byte of memory.
        op_map.insert(0x49, OpCode::new("EOR", 2, 2, AddressingMode::Immediate));
        op_map.insert(0x45, OpCode::new("EOR", 2, 3, AddressingMode::ZeroPage));
        op_map.insert(0x55, OpCode::new("EOR", 2, 4, AddressingMode::ZeroPage_X));
        op_map.insert(0x4d, OpCode::new("EOR", 3, 4, AddressingMode::Absolute));
        op_map.insert(
            0x5d,
            OpCode::new(
                "EOR",
                3,
                4, /* +1 if page crossed */
                AddressingMode::Absolute_X,
            ),
        );
        op_map.insert(
            0x59,
            OpCode::new(
                "EOR",
                3,
                4, /* +1 if page crossed */
                AddressingMode::Absolute_Y,
            ),
        );
        op_map.insert(0x41, OpCode::new("EOR", 2, 6, AddressingMode::Indirect_X));
        op_map.insert(
            0x51,
            OpCode::new(
                "EOR",
                2,
                5, /* +1 if page corssed */
                AddressingMode::Indirect_Y,
            ),
        );

        // ASL: This operation shifts all the bits of the accumulator or memory contents one bit left.
        op_map.insert(
            op::ASL,
            OpCode::new("ASL", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(0x06, OpCode::new("ASL", 2, 5, AddressingMode::ZeroPage));
        op_map.insert(0x16, OpCode::new("ASL", 2, 6, AddressingMode::ZeroPage_X));
        op_map.insert(0x0e, OpCode::new("ASL", 3, 6, AddressingMode::Absolute));
        op_map.insert(
            0x1e,
            OpCode::new(
                "ASL",
                3,
                7, /* +1 if page crossed */
                AddressingMode::Absolute_X,
            ),
        );

        // LSR: Each of the bits in A or M is shift one place to the right. The bit that was in bit 0 is shifted into the carry flag. Bit 7 is set to zero.
        op_map.insert(
            op::LSR,
            OpCode::new("LSR", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(0x46, OpCode::new("LSR", 2, 5, AddressingMode::ZeroPage));
        op_map.insert(0x56, OpCode::new("LSR", 2, 6, AddressingMode::ZeroPage_X));
        op_map.insert(0x4e, OpCode::new("LSR", 3, 6, AddressingMode::Absolute));
        op_map.insert(
            0x5e,
            OpCode::new(
                "LSR",
                3,
                7, /* +1 if page crossed */
                AddressingMode::Absolute_X,
            ),
        );

        // ROL: Move each of the bits in either A or M one place to the left. Bit 0 is filled with the current value of the carry flag whilst the old bit 7 becomes the new carry flag value.
        op_map.insert(
            op::ROL,
            OpCode::new("ROL", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(0x26, OpCode::new("ROL", 2, 5, AddressingMode::ZeroPage));
        op_map.insert(0x36, OpCode::new("ROL", 2, 6, AddressingMode::ZeroPage_X));
        op_map.insert(0x2e, OpCode::new("ROL", 3, 6, AddressingMode::Absolute));
        op_map.insert(
            0x3e,
            OpCode::new(
                "ROL",
                3,
                7, /* +1 if page crossed */
                AddressingMode::Absolute_X,
            ),
        );

        // ROR:
        op_map.insert(
            op::ROR,
            OpCode::new("ROR", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(0x66, OpCode::new("ROR", 2, 5, AddressingMode::ZeroPage));
        op_map.insert(0x76, OpCode::new("ROR", 2, 6, AddressingMode::ZeroPage_X));
        op_map.insert(0x6e, OpCode::new("ROR", 3, 6, AddressingMode::Absolute));
        op_map.insert(
            0x7e,
            OpCode::new(
                "ROR",
                3,
                7, /* +1 if page crossed */
                AddressingMode::Absolute_X,
            ),
        );

        // BIT: This instructions is used to test if one or more bits are set in a target memory location.
        op_map.insert(0x24, OpCode::new("BIT", 2, 3, AddressingMode::ZeroPage));
        op_map.insert(0x2c, OpCode::new("BIT", 3, 4, AddressingMode::Absolute));

        // CMP: This instruction compares the contents of the accumulator with another memory held value and sets the zero and carry flags as appropriate.
        op_map.insert(0xc9, OpCode::new("CMP", 2, 2, AddressingMode::Immediate));
        op_map.insert(0xc5, OpCode::new("CMP", 2, 3, AddressingMode::ZeroPage));
        op_map.insert(0xd5, OpCode::new("CMP", 2, 4, AddressingMode::ZeroPage_X));
        op_map.insert(0xcd, OpCode::new("CMP", 3, 4, AddressingMode::Absolute));
        op_map.insert(
            0xdd,
            OpCode::new(
                "CMP",
                3,
                4, /* +1 if page crossed */
                AddressingMode::Absolute_X,
            ),
        );
        op_map.insert(
            0xd9,
            OpCode::new(
                "CMP",
                3,
                4, /* +1 if page crossed */
                AddressingMode::Absolute_Y,
            ),
        );
        op_map.insert(0xc1, OpCode::new("CMP", 2, 6, AddressingMode::Indirect_X));
        op_map.insert(
            0xd1,
            OpCode::new(
                "CMP",
                2,
                5, /* +1 if page corssed */
                AddressingMode::Indirect_Y,
            ),
        );

        // CPX: This instruction compares the contents of the X register with another memory held value and sets the zero and carry flags as appropriate.
        op_map.insert(0xe0, OpCode::new("CPX", 2, 2, AddressingMode::Immediate));
        op_map.insert(0xe4, OpCode::new("CPX", 2, 3, AddressingMode::ZeroPage));
        op_map.insert(0xec, OpCode::new("CPX", 3, 4, AddressingMode::Absolute));

        // CPY: This instruction compares the contents of the Y register with another memory held value and sets the zero and carry flags as appropriate.
        op_map.insert(0xc0, OpCode::new("CPY", 2, 2, AddressingMode::Immediate));
        op_map.insert(0xc4, OpCode::new("CPY", 2, 3, AddressingMode::ZeroPage));
        op_map.insert(0xcc, OpCode::new("CPY", 3, 4, AddressingMode::Absolute));

        // DEC: Subtracts one from the value held at a specified memory location setting the zero and negative flags as appropriate.
        op_map.insert(0xc6, OpCode::new("DEC", 2, 5, AddressingMode::ZeroPage));
        op_map.insert(0xd6, OpCode::new("DEC", 2, 6, AddressingMode::ZeroPage_X));
        op_map.insert(0xce, OpCode::new("DEC", 3, 6, AddressingMode::Absolute));
        op_map.insert(0xde, OpCode::new("DEC", 3, 7, AddressingMode::Absolute_X));

        // INC: Adds one to the value held at a specified memory location setting the zero and negative flags as appropriate.
        op_map.insert(0xe6, OpCode::new("INC", 2, 5, AddressingMode::ZeroPage));
        op_map.insert(0xf6, OpCode::new("INC", 2, 6, AddressingMode::ZeroPage_X));
        op_map.insert(0xee, OpCode::new("INC", 3, 6, AddressingMode::Absolute));
        op_map.insert(0xfe, OpCode::new("INC", 3, 7, AddressingMode::Absolute_X));

        // JMP: Sets the program counter to the address specified by the operand.
        op_map.insert(0x4c, OpCode::new("JMP", 3, 3, AddressingMode::Absolute));
        op_map.insert(0x6c, OpCode::new("JMP", 3, 5, AddressingMode::Indirect));

        // BCC: If the carry flag is clear then add the relative displacement to the program counter to cause a branch to a new location.
        op_map.insert(0x90, OpCode::new("BCC", 2, 2, AddressingMode::Immediate));

        // BCS: If the carry flag is set then add the relative displacement to the program counter to cause a branch to a new location.
        op_map.insert(0xb0, OpCode::new("BCS", 2, 2, AddressingMode::Immediate));

        // BEQ: If the zero flag is set then add the relative displacement to the program counter to cause a branch to a new location.
        op_map.insert(0xf0, OpCode::new("BEQ", 2, 2, AddressingMode::Immediate));

        // BNE
        op_map.insert(0xd0, OpCode::new("BNE", 2, 2, AddressingMode::Immediate));

        // BPL
        op_map.insert(0x10, OpCode::new("BPL", 2, 2, AddressingMode::Immediate));

        // BVC
        op_map.insert(0x50, OpCode::new("BVC", 2, 2, AddressingMode::Immediate));

        // BVS
        op_map.insert(0x70, OpCode::new("BVS", 2, 2, AddressingMode::Immediate));

        // BMI
        op_map.insert(op::BMI, OpCode::new("BMI", 2, 2, AddressingMode::Immediate));

        op_map.insert(
            op::PHA,
            OpCode::new("PHA", 1, 3, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::PHP,
            OpCode::new("PHP", 1, 3, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::PLA,
            OpCode::new("PLA", 1, 4, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::PLP,
            OpCode::new("PLP", 1, 4, AddressingMode::NoneAddressing),
        );

        op_map.insert(
            op::RTI,
            OpCode::new("RTI", 1, 6, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::RTS,
            OpCode::new("RTS", 1, 6, AddressingMode::NoneAddressing),
        );

        op_map.insert(
            op::JSR,
            OpCode::new("JSR", 3, 6, AddressingMode::Absolute),
        );
        op_map.insert(
            op::TAX,
            OpCode::new("TAX", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::TAY,
            OpCode::new("TAY", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::TSX,
            OpCode::new("TSX", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::TXA,
            OpCode::new("TXA", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::TXS,
            OpCode::new("TXS", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::TYA,
            OpCode::new("TYA", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::SEC,
            OpCode::new("SEC", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::CLC,
            OpCode::new("CLC", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::DEX,
            OpCode::new("DEX", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::DEY,
            OpCode::new("DEY", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::CLI,
            OpCode::new("CLI", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::CLD,
            OpCode::new("CLD", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::CLV,
            OpCode::new("CLV", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::INY,
            OpCode::new("INY", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::INX,
            OpCode::new("INX", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::NOP,
            OpCode::new("NOP", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::SED,
            OpCode::new("SED", 1, 2, AddressingMode::NoneAddressing),
        );
        op_map.insert(
            op::SEI,
            OpCode::new("SEI", 1, 2, AddressingMode::NoneAddressing),
        );

        op_map.insert(
            op::BRK,
            OpCode::new("BRK", 1, 7, AddressingMode::NoneAddressing),
        );

        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            stack_counter: 0,
            bus,
            op_map,
            added_cycles_of_addr: 0,
            added_cycles_of_br: 0,
        }
    }

    pub fn load_and_run(&mut self) {
        // self.load();
        self.reset();
        self.run();
    }

    pub fn mem_read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    pub fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0x24;
        self.program_counter = self.mem_read_u16(0xFFFC);
        eprintln!("program_counter: 0x{:X}", self.program_counter);
        /* [0x0100 .. 0x1ff] */
        self.stack_counter = 0xfd;
    }

    pub fn run(&mut self) {
        self.run_with_callbacks(|_| {});
    }

    pub fn run_with_callbacks<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        loop {
            if self.bus.poll_nmi_status() {
                self.interrupt_nmi();
            }

            callback(self);

            /*if self.program_counter < self.mem_read_u16(0xFFFC) {
                panic!("invalid program_counter:{}", self.program_counter);
            }*/
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let old_pc = self.program_counter;

            if self.op_map.contains_key(&code) {
                let op = self.op_map[&code].clone();
                let mode = &op.mode;
                match op.name.as_str() {
                    "LDA" => {
                        self.lda(&mode);
                    }
                    "LDX" => {
                        self.ldx(&mode);
                    }
                    "LDY" => {
                        self.ldy(&mode);
                    }
                    "STA" => {
                        self.sta(&mode);
                    }
                    "STX" => {
                        self.stx(&mode);
                    }
                    "STY" => {
                        self.sty(&mode);
                    }
                    "ADC" => {
                        self.adc(&mode);
                    }
                    "SBC" => {
                        self.sbc(&mode);
                    }
                    "AND" => {
                        self.and(&mode);
                    }
                    "ORA" => {
                        self.ora(&mode);
                    }
                    "EOR" => {
                        self.eor(&mode);
                    }
                    "ASL" => {
                        self.asl(&mode);
                    }
                    "LSR" => {
                        self.lsr(&mode);
                    }
                    "ROL" => {
                        self.rol(&mode);
                    }
                    "ROR" => {
                        self.ror(&mode);
                    }
                    "BIT" => {
                        self.bit(&mode);
                    }
                    "CMP" => {
                        self.cmp(&mode);
                    }
                    "CPX" => {
                        self.cpx(&mode);
                    }
                    "CPY" => {
                        self.cpy(&mode);
                    }
                    "DEC" => {
                        self.dec(&mode);
                    }
                    "INC" => {
                        self.inc(&mode);
                    }
                    "JMP" => {
                        self.jmp(&mode);
                    }
                    "BCC" => {
                        self.bcc(&mode);
                    }
                    "BCS" => {
                        self.bcs(&mode);
                    }
                    "BEQ" => {
                        self.beq(&mode);
                    }
                    "BMI" => {
                        self.bmi(&mode);
                    }
                    "BNE" => {
                        self.bne(&mode);
                    }
                    "BPL" => {
                        self.bpl(&mode);
                    }
                    "BVC" => {
                        self.bvc(&mode);
                    }
                    "BVS" => {
                        self.bvs(&mode);
                    }
                    "TAX" => {
                        self.tax();
                    }
                    "TAY" => {
                        self.tay();
                    }
                    "TSX" => {
                        self.tsx();
                    }
                    "TXA" => {
                        self.txa();
                    }
                    "TXS" => {
                        self.txs();
                    }
                    "TYA" => {
                        self.tya();
                    }
                    "SEC" => {
                        self.sec();
                    }
                    "CLC" => {
                        self.clc();
                    }
                    "DEX" => {
                        self.dex();
                    }
                    "DEY" => {
                        self.dey();
                    }
                    "CLI" => {
                        self.cli();
                    }
                    "CLD" => {
                        self.cld();
                    }
                    "CLV" => {
                        self.clv();
                    }
                    "INY" => {
                        self.iny();
                    }
                    "INX" => {
                        self.inx();
                    }
                    "NOP" => {
                        self.nop();
                    }
                    "PHA" => {
                        self.pha();
                    }
                    "PHP" => {
                        self.php();
                    }
                    "PLA" => {
                        self.pla();
                    }
                    "PLP" => {
                        self.plp();
                    }
                    "RTI" => {
                        self.rti();
                    }
                    "JSR" => {
                        self.jsr();
                    }
                    "RTS" => {
                        self.rts();
                    }
                    "SED" => {
                        self.sed();
                    }
                    "SEI" => {
                        self.sei();
                    }
                    "BRK" => {
                        return;
                    }
                    _ => {
                        panic!("Internal error in op_map match~");
                    }
                }

                // propagate tick to bus
                self.bus
                    .tick(op.cycles + self.added_cycles_of_br + self.added_cycles_of_addr);
                self.added_cycles_of_addr = 0;
                self.added_cycles_of_br = 0;

                if self.program_counter == old_pc {
                    self.program_counter += (op.op_length - 1) as u16;
                }
                continue;
            }

            // single address mode
            match code {
                _ => {
                    println!("{:x}", code);
                    todo!()
                }
            }
        }
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_y = value;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }

    fn add_rega_and_value(&mut self, value: u8) {
        let (result, carry) = self.register_a.overflowing_add(value);
        if carry {
            StatusFlag::Carry.add(&mut self.status);
        } else {
            StatusFlag::Carry.remove(&mut self.status);
        }

        let (_, overflowed) = (self.register_a as i8).overflowing_add(value as i8);
        if overflowed {
            StatusFlag::Overflow.add(&mut self.status);
        } else {
            StatusFlag::Overflow.remove(&mut self.status);
        }
        self.register_a = result;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.add_rega_and_value(value);
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.add_rega_and_value(!value + 1);
    }

    /*
    fn sbc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let (result, carry) = self.register_a.overflowing_sub(value);
        if carry {
            self.status = self.status & StatusFlag::Carry.reverse();
        } else {
            self.status = self.status | StatusFlag::Carry;
        }

        let (_, overflowed) = (self.register_a as i8).overflowing_sub(value as i8);
        if overflowed {
            self.status = self.status | StatusFlag::Overflow;
        } else {
            self.status = self.status & StatusFlag::Overflow.reverse();
        }
        self.register_a = result;
        self.update_zero_and_negative_flags(self.register_a);
    }*/

    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let result = self.register_a & value;
        self.register_a = result;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let result = self.register_a | value;
        self.register_a = result;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let result = self.register_a ^ value;
        self.register_a = result;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn asl(&mut self, mode: &AddressingMode) {
        if mode == &AddressingMode::NoneAddressing {
            self.asl_accumulate();
            return;
        }
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let carry: bool = (value & 0x80) > 0;
        let result = value << 1;

        if carry {
            StatusFlag::Carry.add(&mut self.status);
        } else {
            StatusFlag::Carry.remove(&mut self.status);
        }
        self.mem_write(addr, result);
        self.update_zero_and_negative_flags(result);
    }

    fn asl_accumulate(&mut self) {
        let carry: bool = (self.register_a & 0x80) > 0;
        let result = self.register_a << 1;

        if carry {
            StatusFlag::Carry.add(&mut self.status);
        } else {
            StatusFlag::Carry.remove(&mut self.status);
        }

        self.register_a = result;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn cal_displacement_and_add_cycles(&mut self, mode: &AddressingMode) -> (u8, u16) {
        let addr = self.get_operand_address(&mode);
        let value = self.mem_read(addr);
        let result = ((self.program_counter as i16) + ((value as i8) as i16)) as u16 + 1;

        // the '1' added to program_counter is the second byte of branch instruction
        let mut addr_cycles = 0;
        if (self.program_counter + 1) & 0xff00 != result & 0xff00 {
            // +1 for cross page
            addr_cycles += 1;
        }
        (addr_cycles, result)
    }

    fn branch_jump(&mut self, mode: &AddressingMode) {
        let (addr_cycles, dest) = self.cal_displacement_and_add_cycles(&mode);
        self.added_cycles_of_addr += addr_cycles;

        // +1 for the successful branch
        self.added_cycles_of_br += 1;
        self.program_counter = dest;
    }

    fn bcc(&mut self, mode: &AddressingMode) {
        if !StatusFlag::Carry.among(self.status) {
            self.branch_jump(&mode);
        }
    }

    fn bcs(&mut self, mode: &AddressingMode) {
        if StatusFlag::Carry.among(self.status) {
            self.branch_jump(&mode);
        }
    }

    fn beq(&mut self, mode: &AddressingMode) {
        if StatusFlag::Zero.among(self.status) {
            self.branch_jump(&mode);
        }
    }

    fn bne(&mut self, mode: &AddressingMode) {
        if !StatusFlag::Zero.among(self.status) {
            self.branch_jump(&mode);
        }
    }

    fn bmi(&mut self, mode: &AddressingMode) {
        if StatusFlag::Negative.among(self.status) {
            self.branch_jump(&mode);
        }
    }

    fn bpl(&mut self, mode: &AddressingMode) {
        if !StatusFlag::Negative.among(self.status) {
            self.branch_jump(&mode);
        }
    }

    fn bvc(&mut self, mode: &AddressingMode) {
        if !StatusFlag::Overflow.among(self.status) {
            self.branch_jump(&mode);
        }
    }

    fn bvs(&mut self, mode: &AddressingMode) {
        if StatusFlag::Overflow.among(self.status) {
            self.branch_jump(&mode);
        }
    }

    fn lsr(&mut self, mode: &AddressingMode) {
        if mode == &AddressingMode::NoneAddressing {
            self.lsr_accumulate();
            return;
        }
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let carry: bool = (value & 0x1u8) > 0;
        let result = value >> 1;

        if carry {
            StatusFlag::Carry.add(&mut self.status);
        } else {
            StatusFlag::Carry.remove(&mut self.status);
        }
        self.mem_write(addr, result);
        self.update_zero_and_negative_flags(result);
    }

    fn lsr_accumulate(&mut self) {
        let carry: bool = (self.register_a & 0x1u8) > 0;
        let result = self.register_a >> 1;

        if carry {
            StatusFlag::Carry.add(&mut self.status);
        } else {
            StatusFlag::Carry.remove(&mut self.status);
        }
        self.register_a = result;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn rol(&mut self, mode: &AddressingMode) {
        if mode == &AddressingMode::NoneAddressing {
            self.rol_accumulate();
            return;
        }
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let carry: bool = (value & 0x80) > 0;
        let mut result = value << 1;

        if StatusFlag::Carry.test(self.status) {
            result |= 1u8;
        }
        if carry {
            StatusFlag::Carry.add(&mut self.status);
        } else {
            StatusFlag::Carry.remove(&mut self.status);
        }
        self.mem_write(addr, result);
        self.update_zero_and_negative_flags(result);
    }

    fn rol_accumulate(&mut self) {
        let carry: bool = (self.register_a & 0x80) > 0;
        let mut result = self.register_a << 1;

        if StatusFlag::Carry.test(self.status) {
            result |= 1u8;
        }
        if carry {
            StatusFlag::Carry.add(&mut self.status);
        } else {
            StatusFlag::Carry.remove(&mut self.status);
        }
        self.register_a = result;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ror(&mut self, mode: &AddressingMode) {
        if mode == &AddressingMode::NoneAddressing {
            self.ror_accumulate();
            return;
        }
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let carry: bool = (value & 0x1) > 0;
        let mut result = value >> 1;

        if StatusFlag::Carry.test(self.status) {
            result |= 0x80u8;
        }
        if carry {
            StatusFlag::Carry.add(&mut self.status);
        } else {
            StatusFlag::Carry.remove(&mut self.status);
        }
        self.mem_write(addr, result);
        self.update_zero_and_negative_flags(result);
    }

    fn ror_accumulate(&mut self) {
        let carry: bool = (self.register_a & 0x1) > 0;
        let mut result = self.register_a >> 1;
        if StatusFlag::Carry.test(self.status) {
            result |= 0x80u8;
        }
        if carry {
            StatusFlag::Carry.add(&mut self.status);
        } else {
            StatusFlag::Carry.remove(&mut self.status);
        }
        self.register_a = result;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        // Negative
        if value & 0x80 > 0 {
            StatusFlag::Negative.add(&mut self.status);
        } else {
            StatusFlag::Negative.remove(&mut self.status);
        }

        // Overflow
        if value & 0x40 > 0 {
            StatusFlag::Overflow.add(&mut self.status);
        } else {
            StatusFlag::Overflow.remove(&mut self.status);
        }

        if value & self.register_a == 0 {
            StatusFlag::Zero.add(&mut self.status);
        } else {
            StatusFlag::Zero.remove(&mut self.status);
        }
    }

    fn cmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        if self.register_a >= value {
            StatusFlag::Carry.add(&mut self.status);
        } else {
            StatusFlag::Carry.remove(&mut self.status);
        }
        if self.register_a == value {
            StatusFlag::Zero.add(&mut self.status);
        } else {
            StatusFlag::Zero.remove(&mut self.status);
        }

        if self.register_a & 0x80 == value & 0x80 {
            StatusFlag::Negative.remove(&mut self.status);
        } else {
            StatusFlag::Negative.add(&mut self.status);
        }
    }

    fn cpx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        if self.register_x >= value {
            StatusFlag::Carry.add(&mut self.status);
        } else {
            StatusFlag::Carry.remove(&mut self.status);
        }
        if self.register_x == value {
            StatusFlag::Zero.add(&mut self.status);
        } else {
            StatusFlag::Zero.remove(&mut self.status);
        }
        if self.register_x & 0x80 == value & 0x80 {
            StatusFlag::Negative.remove(&mut self.status);
        } else {
            StatusFlag::Negative.add(&mut self.status);
        }
    }

    fn cpy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        if self.register_y >= value {
            StatusFlag::Carry.add(&mut self.status);
        } else {
            StatusFlag::Carry.remove(&mut self.status);
        }
        if self.register_y == value {
            StatusFlag::Zero.add(&mut self.status);
        } else {
            StatusFlag::Zero.remove(&mut self.status);
        }
        if self.register_y & 0x80 == value & 0x80 {
            StatusFlag::Negative.remove(&mut self.status);
        } else {
            StatusFlag::Negative.add(&mut self.status);
        }
    }

    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let (result, _) = value.overflowing_sub(1);
        self.mem_write(addr, result);
        self.update_zero_and_negative_flags(result);
    }

    fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        // todo: should `as i8` ?
        let (result, _) = value.overflowing_add(1);
        self.mem_write(addr, result);
        self.update_zero_and_negative_flags(result);
    }

    fn jsr(&mut self) {
        let addr = self.get_operand_address(&AddressingMode::Absolute);
        // self.program_counter + 2 -1
        self.push_u16(self.program_counter + 1);
        self.program_counter = addr;
    }

    fn nop(&mut self) {
        // do nothing
    }

    fn pha(&mut self) {
        self.push(self.register_a);
    }

    fn php(&mut self) {
        let mut status = self.status;
        StatusFlag::BreakCommand.add(&mut status);
        self.push(status);
    }

    fn pla(&mut self) {
        self.register_a = self.pop();
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn plp(&mut self) {
        self.status = self.pop();
        // FIXME: from nestest, status should remove BreakCommand
        StatusFlag::BreakCommand.remove(&mut self.status);
        StatusFlag::Undefined.add(&mut self.status);
    }

    fn rti(&mut self) {
        self.status = self.pop();

        StatusFlag::BreakCommand.remove(&mut self.status);

        // todo: need verify from spec
        self.program_counter = self.pop_u16();
    }

    fn rts(&mut self) {
        // todo: need verify from spec
        self.program_counter = self.pop_u16();
        self.program_counter += 1;
    }

    fn jmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        if *mode == AddressingMode::Absolute {
            self.program_counter = addr;
        } else {
            assert!(*mode == AddressingMode::Indirect);
            self.program_counter = self.mem_read_u16(addr);
        }
    }

    fn inx(&mut self) {
        let (result, _) = self.register_x.overflowing_add(1);
        self.register_x = result;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn iny(&mut self) {
        let (result, _) = self.register_y.overflowing_add(1);
        self.register_y = result;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn dex(&mut self) {
        let (result, _) = self.register_x.overflowing_sub(1);
        self.register_x = result;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn dey(&mut self) {
        let (result, _) = self.register_y.overflowing_sub(1);
        self.register_y = result;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn tya(&mut self) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tsx(&mut self) {
        self.register_x = self.stack_counter;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn txs(&mut self) {
        self.stack_counter = self.register_x;
    }

    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn cld(&mut self) {
        StatusFlag::DecimalMode.remove(&mut self.status);
    }

    // Clears the interrupt disable flag allowing normal interrupt requests to be serviced.
    fn cli(&mut self) {
        StatusFlag::InterruptDisable.remove(&mut self.status);
    }

    // Clears the overflow flag.
    fn clv(&mut self) {
        StatusFlag::Overflow.remove(&mut self.status);
    }

    // set the carry flag to one.
    fn sec(&mut self) {
        StatusFlag::Carry.add(&mut self.status);
    }

    fn sed(&mut self) {
        StatusFlag::DecimalMode.add(&mut self.status);
    }

    fn sei(&mut self) {
        StatusFlag::InterruptDisable.add(&mut self.status);
    }

    // set the carry flag to zero.
    fn clc(&mut self) {
        StatusFlag::Carry.remove(&mut self.status);
    }

    fn brk(&mut self) {
        self.push_u16(self.program_counter);
        self.push(self.status);

        // todo: force interrupt
        let value = self.mem_read_u16(0xfffe);
        self.program_counter = value;

        StatusFlag::BreakCommand.add(&mut self.status);

        // side effect
        StatusFlag::InterruptDisable.add(&mut self.status);
        // StatusFlag::DecimalMode.remove(&mut self.status);
    }

    fn push_u16(&mut self, value: u16) {
        let lo: u8 = (value & 0xff) as u8;
        let hi: u8 = ((value >> 8) & 0xff) as u8;
        self.push(lo);
        self.push(hi);
    }

    fn push(&mut self, value: u8) {
        self.mem_write(self.stack_counter as u16 + 0x100, value);
        let (result, overflow) = self.stack_counter.overflowing_sub(1);
        self.stack_counter = result;
        if overflow {
            panic!("overflow at push");
        }
    }

    fn pop_u16(&mut self) -> u16 {
        let mut result: u16;
        result = self.pop() as u16;
        result <<= 8;
        result |= self.pop() as u16;
        return result;
    }

    fn pop(&mut self) -> u8 {
        let (result, overflow) = self.stack_counter.overflowing_add(1);
        self.stack_counter = result;
        if overflow {
            panic!("overflow at pop");
        }
        let value = self.mem_read(self.stack_counter as u16 + 0x100);
        return value;
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status = self.status | StatusFlag::Zero;
        } else {
            self.status = self.status & StatusFlag::Zero.reverse();
        }

        if result & 0b1000_0000 != 0 {
            self.status = self.status | StatusFlag::Negative;
        } else {
            self.status = self.status & StatusFlag::Negative.reverse();
        }
    }

    fn interrupt_nmi(&mut self) {
        self.push_u16(self.program_counter);
        StatusFlag::BreakCommand.add(&mut self.status);

        let mut flag = self.status.clone();
        StatusFlag::BreakCommand.add(&mut flag);

        self.push(flag);
        StatusFlag::InterruptDisable.add(&mut self.status);

        self.bus.tick(2);
        self.program_counter = self.mem_read_u16(0xfffa);
    }

    pub fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,

            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,

            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }

            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }

            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                if base & 0xff00 != addr & 0xff00 {
                    self.added_cycles_of_addr += 1;
                }
                addr
            }

            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                if base & 0xff00 != addr & 0xff00 {
                    self.added_cycles_of_addr += 1;
                }
                addr
            }

            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);

                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }

            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);

                // this is similar to Absolute_Y
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                if deref_base & 0xff00 != deref & 0xff00 {
                    self.added_cycles_of_addr += 1;
                }
                deref
            }

            AddressingMode::Indirect => {
                todo!();
            }
            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    pub fn trace(&mut self) -> String {
        let pc = self.program_counter;
        let mut res = format!("{:0>4X}  ", pc);

        let op = self.mem_read(pc);
        let opcode_v = &self.op_map.get(&op);
        if opcode_v.is_none() {
            eprintln!("opcode {:X} no implemented! ", op);
        }
        let opcode = opcode_v.unwrap();

        let name = &opcode.name.clone();
        let op_length = opcode.op_length;
        let mode = opcode.mode;

        let mut code_res = String::from("");
        for i in 0..op_length {
            if i > 0 {
                code_res.push(' ');
            }
            code_res.push_str(&format!("{:02X}", self.mem_read(pc + (i as u16))));
        }
        while code_res.len() < 10 {
            code_res.push(' ');
        }

        res.push_str(format!("{}{} ", code_res, name).as_str());

        let mut addr_res = String::from("");
        self.program_counter += 1;
        match mode {
            AddressingMode::Absolute => {
                let addr = self.get_operand_address(&mode);
                addr_res.push_str(&format!("${:04X}", addr));
            }
            AddressingMode::ZeroPage => {
                let addr = self.get_operand_address(&mode);
                addr_res.push_str(&format!("${:02X} = {:02X}", addr, self.mem_read(addr)));
            }
            AddressingMode::Immediate => {
                let addr = self.get_operand_address(&mode);
                addr_res.push_str(&format!("#${:02X}", self.mem_read(addr)));
            }
            AddressingMode::Indirect_X => {
                let addr = self.get_operand_address(&mode);
                let p_addr = self.mem_read(self.program_counter);
                let r_addr = p_addr.wrapping_add(self.register_x);
                addr_res.push_str(&format!(
                    "(${:02X},X) @ {:02X} = {:02X}{:02X} = {:02X}",
                    p_addr,
                    r_addr,
                    r_addr + 1,
                    r_addr,
                    self.mem_read(addr)
                ));
            }
            AddressingMode::Indirect_Y => {
                let addr = self.get_operand_address(&mode);
                let base = self.mem_read(self.program_counter);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                if deref_base & 0xff00 != deref & 0xff00 {
                    self.added_cycles_of_addr += 1;
                }

                addr_res.push_str(&format!(
                    "(${:02X}),Y = {:02X}{:02X} @ {:02X}{:02X} = {:02X}",
                    base,
                    deref >> 8,
                    deref & 0xff,
                    deref >> 8,
                    deref & 0xff,
                    self.mem_read(addr)
                ));
            }
            AddressingMode::NoneAddressing => {
                // empty
            }
            _ => {
                let addr = self.get_operand_address(&mode);
                addr_res.push_str(format!("${:X}", addr).as_str());
                // panic!("never come here");
            }
        };
        match op {
            op::BCC | op::BCS | op::BEQ | op::BNE | op::BPL | op::BVC | op::BVS | op::BMI => {
                let (_, dest) = self.cal_displacement_and_add_cycles(&mode);
                let dest_str = format!("${:02X}", dest);
                res.push_str(&format!("{:<28}", dest_str));
            } 
            _ => {
                res.push_str(&format!("{:<28}", addr_res));
            }
        }
        self.program_counter -= 1;
        res.push_str(&format!(
            "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            self.register_a, self.register_x, self.register_y, self.status, self.stack_counter
        ));
        res
    }

    pub fn negative(&self) -> bool {
        return StatusFlag::Negative.among(self.status);
    }
    pub fn zero(&self) -> bool {
        return StatusFlag::Zero.among(self.status);
    }
    pub fn overflow(&self) -> bool {
        return StatusFlag::Overflow.among(self.status);
    }
}

pub const LDA_IMMEDIATE: u8 = 0xa9u8;
pub const LDA_ZEROPAGE: u8 = 0xa5u8;
pub const LDA_ABSOLUTE: u8 = 0xadu8;

pub const LDX_IMMEDIATE: u8 = 0xa2u8;
pub const LDY_IMMEDIATE: u8 = 0xa0u8;

pub const STA_ZEROPAGE: u8 = 0x85u8;
pub const STA_ABSOLUTE: u8 = 0x8du8;

pub const ADC_IMMEDIATE: u8 = 0x69u8;

// set carry flag
pub const SEC: u8 = 0x38u8;

// clear carry flag
pub const CLC: u8 = 0x18u8;
