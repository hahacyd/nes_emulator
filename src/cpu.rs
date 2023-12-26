use std::collections::HashMap;

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
    memory: [u8; 0xFFFF],

    op_map: HashMap<u8, OpCode>,
}

#[derive(Debug, Clone)]
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

        // EOR:
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
            0x0a,
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
            0x4a,
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
            0x2a,
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
            0x6a,
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

        let mut op_map: HashMap<u8, OpCode> = HashMap::new();
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0u8; 0xFFFF],
            op_map,
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
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
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        // set where address the program start
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn run(&mut self) {
        loop {
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;

            if self.op_map.contains_key(&code) {
                let op = self.op_map[&code].clone();
                // self.program_counter += (op.op_length - 1) as u16;
                continue;
            }
            match code {
                0xE8 => self.inx(),
                0xAA => self.tax(),
                0x00 => {
                    return;
                }
                _ => todo!(),
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

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn inx(&mut self) {
        let (result, overflowed) = self.register_x.overflowing_add(1);
        self.register_x = result;
        print!("{}", self.register_x);
        self.update_zero_and_negative_flags(self.register_x);

        if overflowed {
            self.status = self.status | 0b0100_0000;
        } else {
            self.status = self.status & 0b1011_1111;
        }
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }

        if result & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xe8_inx_increment_x_register() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xe8, 0x00]);
        assert_eq!(cpu.register_x, 0x1);
        assert!(cpu.status & 0b0000_0010 == 0x0);
        assert!(cpu.status & 0b1000_0010 == 0x0);
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0xff, 0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }
}
