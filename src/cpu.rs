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
    
    lda_op_map: HashMap<u8, OpCode>,
    ldx_op_map: HashMap<u8, OpCode>,
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
        // lda
        let mut lda_op_map: HashMap<u8, OpCode> = HashMap::new();
        lda_op_map.insert(0xa9, OpCode::new("LDA", 2, 2, AddressingMode::Immediate));
        lda_op_map.insert(0xa5, OpCode::new("LDA", 2, 3, AddressingMode::ZeroPage));
        lda_op_map.insert(0xb5, OpCode::new("LDA", 2, 4, AddressingMode::ZeroPage_X));
        lda_op_map.insert(0xad, OpCode::new("LDA", 3, 4, AddressingMode::Absolute));
        lda_op_map.insert(0xbd, OpCode::new("LDA", 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_X));
        lda_op_map.insert(0xb9, OpCode::new("LDA", 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_Y));
        lda_op_map.insert(0xa1, OpCode::new("LDA", 2, 6, AddressingMode::Indirect_X));
        lda_op_map.insert(0xb1, OpCode::new("LDA", 2, 5 /* +1 if page corssed */, AddressingMode::Indirect_Y));

        // ldx
        let mut ldx_op_map: HashMap<u8, OpCode> = HashMap::new();
        ldx_op_map.insert(0xa2, OpCode::new("LDX", 2, 2, AddressingMode::Immediate));
        ldx_op_map.insert(0xa6, OpCode::new("LDX", 2, 3, AddressingMode::ZeroPage));
        ldx_op_map.insert(0xb6, OpCode::new("LDX", 2, 4, AddressingMode::ZeroPage_Y));
        ldx_op_map.insert(0xae, OpCode::new("LDX", 3, 4, AddressingMode::Absolute));
        ldx_op_map.insert(0xbe, OpCode::new("LDX", 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_Y));

        let mut op_map: HashMap<u8, OpCode> = HashMap::new();
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0u8; 0xFFFF],
            lda_op_map,
            ldx_op_map,
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

            match code {
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                    let op = self.lda_op_map[&code].clone();
                    self.lda(&op.mode);
                    self.program_counter += (op.op_length - 1) as u16;
                }
                0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => {
                    let op = self.ldx_op_map[&code].clone();
                    self.ldx(&op.mode);
                    self.program_counter += (op.op_length - 1) as u16;
                }
                0xE8 => self.inx(),
                0xAA => self.tax(),
                0x00 => {
                    return;
                }
                _ => todo!()
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
