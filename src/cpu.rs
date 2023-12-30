use std::collections::HashMap;
mod op_test;
mod op;

#[derive(Clone)]
struct OpCode {
    name: String,
    op_length: u8,
    cycles: u8,
    mode: AddressingMode,
}

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    pub stack_counter: u8,
    memory: [u8; 0x10000],

    op_map: HashMap<u8, OpCode>,
}

#[derive(Debug, Clone, PartialEq)]
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
    Interrupt = 0b0000_0100,
    DecimalMode = 0b0000_1000,
    BreakCommand = 0b0001_0000,
    Overflow = 0b0010_0000,
    Negative = 0b0100_0000,
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

impl CPU {
    pub fn new() -> Self {
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

        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            stack_counter: 0,
            memory: [0u8; 0x10000],
            op_map,
        }
    }

    pub fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
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
        self.status = 0;
        self.program_counter = self.mem_read_u16(0xFFFC);
        /* [0x0100 .. 0x1ff] */
        self.stack_counter = 0xff;
    }

    pub fn load(&mut self, program: Vec<u8>) {
        // normal address
        // self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);


        // greedy snake
        self.memory[0x0600..(0x0600 + program.len())].copy_from_slice(&program[..]);
        // set where address the program start
        self.mem_write_u16(0xFFFC, 0x0600);
    }

    pub fn run(&mut self) {
        self.run_with_callbacks(|_|{});
    }

    pub fn run_with_callbacks<F>(&mut self, mut callback: F) where F: FnMut(&mut CPU) {
        loop {
            callback(self);
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            if self.op_map.contains_key(&code) {
                let op = self.op_map[&code].clone();
                // self.program_counter += (op.op_length - 1) as u16;
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
                    _ => {
                        panic!("Internal error in op_map match~");
                    }
                }
                if op.name != "JMP" {
                    self.program_counter += (op.op_length - 1) as u16;
                }
                continue;
            }

            // single address mode
            match code {
                op::TAX => self.tax(),
                op::TAY => self.tay(),
                op::TSX => self.tsx(),
                op::TXA => self.txa(),
                op::TXS => self.txs(),
                op::TYA => self.tya(),
                op::SEC => self.sec(),
                op::CLC => self.clc(),
                0x0a => self.asl_accumulate(),
                0x4a => self.lsr_accumulate(),
                0x2a => self.rol_accumulate(),
                0x6a => self.ror_accumulate(),
                op::DEX => self.dex(),
                op::DEY => self.dey(),
                op::CLI => self.cli(),
                op::CLD => self.cld(),
                op::CLV => self.clv(),
                op::INY => self.iny(),
                op::INX => self.inx(),
                op::NOP => self.nop(),
                op::PHA => self.pha(),
                op::PHP => self.php(),
                op::PLA => self.pla(),
                op::PLP => self.plp(),
                op::RTI => self.rti(),
                op::JSR => self.jsr(),
                op::RTS => self.rts(),
                op::SED => self.sed(),
                op::SEI => self.sei(),
                // op::BRK => self.brk(),
                0x00 => {
                    return;
                }
                _ => {
                    println!("{:x}", code);
                    todo!()
                }
            }
            /*if code != op::JSR && code != op::RTI {
                self.program_counter += 1;
            }*/
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

    fn bcc(&mut self, mode: &AddressingMode) {
        if !StatusFlag::Carry.among(self.status) {
            let addr = self.get_operand_address(&mode);
            let value = self.mem_read(addr);
            self.program_counter = ((self.program_counter as i16) + ((value as i8) as i16)) as u16;
        }
    }

    fn bcs(&mut self, mode: &AddressingMode) {
        if StatusFlag::Carry.among(self.status) {
            let addr = self.get_operand_address(&mode);
            let value = self.mem_read(addr);
            self.program_counter = ((self.program_counter as i16) + ((value as i8) as i16)) as u16;
        }
    }

    fn beq(&mut self, mode: &AddressingMode) {
        if StatusFlag::Zero.among(self.status) {
            let addr = self.get_operand_address(&mode);
            let value = self.mem_read(addr);
            self.program_counter = ((self.program_counter as i16) + ((value as i8) as i16)) as u16;
        }
    }

    fn bne(&mut self, mode: &AddressingMode) {
        if !StatusFlag::Zero.among(self.status) {
            let addr = self.get_operand_address(&mode);
            let value = self.mem_read(addr);
            self.program_counter = ((self.program_counter as i16) + ((value as i8) as i16)) as u16;
        }
    }

    fn bmi(&mut self, mode: &AddressingMode) {
        if StatusFlag::Negative.among(self.status) {
            let addr = self.get_operand_address(&mode);
            let value = self.mem_read(addr);
            self.program_counter = ((self.program_counter as i16) + ((value as i8) as i16)) as u16;
        }
    }

    fn bpl(&mut self, mode: &AddressingMode) {
        if !StatusFlag::Negative.among(self.status) {
            let addr = self.get_operand_address(&mode);
            let value = self.mem_read(addr);
            self.program_counter = ((self.program_counter as i16) + ((value as i8) as i16)) as u16;
        }
    }

    fn bvc(&mut self, mode: &AddressingMode) {
        if !StatusFlag::Overflow.among(self.status) {
            let addr = self.get_operand_address(&mode);
            let value = self.mem_read(addr);
            self.program_counter = ((self.program_counter as i16) + ((value as i8) as i16)) as u16;
        }
    }

    fn bvs(&mut self, mode: &AddressingMode) {
        if StatusFlag::Overflow.among(self.status) {
            let addr = self.get_operand_address(&mode);
            let value = self.mem_read(addr);
            self.program_counter = ((self.program_counter as i16) + ((value as i8) as i16)) as u16;
        }
    }

    fn lsr(&mut self, mode: &AddressingMode) {
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
    }

    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut value = self.mem_read(addr);
        // todo: should `as i8` ?
        value = ((value as i8) - 1) as u8;
        self.mem_write(addr, value);
        self.update_zero_and_negative_flags(value);
    }

    fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut value = self.mem_read(addr);
        // todo: should `as i8` ?
        value = ((value as i8) + 1) as u8;
        self.mem_write(addr, value);
        self.update_zero_and_negative_flags(value);
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
    }

    fn rti(&mut self) {
        self.status = self.pop();
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
        StatusFlag::Interrupt.remove(&mut self.status);
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
        StatusFlag::Interrupt.add(&mut self.status);
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
        StatusFlag::Interrupt.add(&mut self.status);
        // StatusFlag::DecimalMode.remove(&mut self.status);
    }

    fn push_u16(&mut self, value: u16) {
        let lo:u8 = (value & 0xff) as u8;
        let hi:u8 = ((value >> 8) & 0xff) as u8;
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
        let mut result:u16;
        result = self.pop() as u16;
        result <<= 8;
        result |= self.pop() as u16;
        return result;
    }

    fn pop(&mut self) -> u8 {
        let value = self.mem_read(self.stack_counter as u16 + 0x100);
        let (result, overflow) = self.stack_counter.overflowing_add(1);
        self.stack_counter = result;
        if overflow {
            panic!("overflow at pop");
        }
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

    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
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
                addr
            }

            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
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

                //todo: check let ptr: u8 = (base as u8).wrapping_add(self.register_y);
                // this is similar to Absolute_Y
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
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
