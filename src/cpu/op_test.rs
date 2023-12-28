#[cfg(test)]
use super::*;

#[test]
fn test_0xa9_lda_immediate_load_data() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
    assert_eq!(cpu.register_a, 0x05);
    assert!(cpu.status & StatusFlag::Zero == 0);
    assert!(cpu.status & StatusFlag::Negative == 0);
}

#[test]
fn test_0xa9_lda_zero_flag() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
    assert!(cpu.status & StatusFlag::Zero == StatusFlag::Zero);
}

#[test]
fn test_0xe8_inx_increment_x_register() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xe8, 0x00]);
    assert_eq!(cpu.register_x, 0x1);
    assert!(cpu.status & StatusFlag::Zero == 0x0);
    assert!(cpu.status & StatusFlag::Negative == 0x0);
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

#[test]
fn test_adc() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x69, 0x01, 0x69, 0x02, 0x00]);
    assert_eq!(cpu.register_a, 3);
}

#[test]
fn test_adc_0x80() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x69, 0x80, 0x00]);
    assert_eq!(cpu.register_a, 0x80);
}

#[test]
fn test_adc_overflow_and_carry() {
    let mut cpu = CPU::new();
    // test carry
    cpu.load_and_run(vec![0x69, 0xff, 0x69, 0x80, 0x00]);
    assert_eq!(cpu.status & StatusFlag::Carry, StatusFlag::Carry);
    assert_eq!(cpu.register_a, 0x7f);

    cpu = CPU::new();
    // test overflow with signed
    cpu.load_and_run(vec![0x69, 0x7f, 0x69, 0x01, 0x00]);
    assert_eq!(cpu.status & StatusFlag::Overflow, StatusFlag::Overflow);
    assert_eq!(cpu.register_a, 0x80);
}

#[test]
fn test_sbc() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x69, 0x02,/* 2 */ 0xe9, 0x01, 0x00]);
    assert_eq!(cpu.register_a, 1);
    assert_eq!(cpu.status & StatusFlag::Carry, StatusFlag::Carry);
    assert_eq!(cpu.status & StatusFlag::Overflow, 0);
}

#[test]
fn test_sbc_overflow_and_carry() {
    let mut cpu = CPU::new();
    // test carry: if overflow with unsigned, clear carry flag
    // todo: does sbc perform signed minus?
    cpu.load_and_run(vec![0xe9, 0x01, 0x00]);
    assert_eq!(cpu.status & StatusFlag::Carry, 0);
    assert_eq!(cpu.register_a, 255);

    cpu = CPU::new();
    // test overflow with signed
    cpu.load_and_run(vec![0x69, 0x7f, /* 0x7f */ 0xe9, 0xff, 0x00]);
    assert_eq!(cpu.status & StatusFlag::Overflow, StatusFlag::Overflow);
    assert_eq!(cpu.register_a, 0x80);
}

#[test]
fn test_and() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x69, 0x3, 0x29, 0x2, 0x00]);
    assert_eq!(cpu.register_a, 2);
}

#[test]
fn test_ora() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x69, 0x1, 0x09, 0x2, 0x00]);
    assert_eq!(cpu.register_a, 3);
}

#[test]
fn test_eor() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x69, 0x4, 0x49, 0x2, 0x00]);
    assert_eq!(cpu.register_a, 6);
}

#[test]
fn test_asl() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x69, 0x6, 0x0a, 0x00]);
    assert_eq!(cpu.register_a, 12);
}

#[test]
fn test_asl_with_carry() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x69, 0x80, 0x0a, 0x00]);
    assert_eq!(cpu.register_a, 0);
    assert!(StatusFlag::Carry.among(cpu.status));
    assert!(StatusFlag::Zero.among(cpu.status));
    assert!(!StatusFlag::Negative.among(cpu.status));
}

#[test]
fn test_lsr() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x69, 0x4, 0x4a, 0x00]);
    assert_eq!(cpu.register_a, 2);
}

#[test]
fn test_lsr_with_carry() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x69, 0x1, 0x4a, 0x00]);
    assert_eq!(cpu.register_a, 0);
    assert!(StatusFlag::Carry.among(cpu.status));
    assert!(StatusFlag::Zero.among(cpu.status));
}

#[test]
fn test_rol() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x69, 0x80, 0x38, /* set carry flag */ 0x2a, 0x00]);
    assert_eq!(cpu.register_a, 1);
    assert!(StatusFlag::Carry.among(cpu.status));
    assert!(!StatusFlag::Zero.among(cpu.status));
    assert!(!StatusFlag::Negative.among(cpu.status));

    cpu.load_and_run(vec![0x69, 0x80, 0x18, /* remove carry flag */ 0x2a, 0x00]);
    assert_eq!(cpu.register_a, 0);
    assert!(StatusFlag::Carry.among(cpu.status));
    assert!(StatusFlag::Zero.among(cpu.status));
    assert!(!StatusFlag::Negative.among(cpu.status));
}

#[test]
fn test_ror() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x69, 0x1, 0x38, /* set carry flag */ 0x6a, 0x00]);
    assert_eq!(cpu.register_a, 0x80);
    assert!(StatusFlag::Carry.among(cpu.status));
    assert!(!StatusFlag::Zero.among(cpu.status));
    assert!(StatusFlag::Negative.among(cpu.status));

    cpu.load_and_run(vec![0x69, 0x1, 0x18, /* set carry flag */ 0x6a, 0x00]);
    assert_eq!(cpu.register_a, 0x0);
    assert!(StatusFlag::Carry.among(cpu.status));
    assert!(StatusFlag::Zero.among(cpu.status));
    assert!(!StatusFlag::Negative.among(cpu.status));
}

#[test]
fn test_bit() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![LDA_IMMEDIATE, 0xc0, STA_ZEROPAGE, 0x00, LDA_IMMEDIATE, 0x3, 0x24 /* bit */, 0x00, 0x00]);
    assert!(StatusFlag::Zero.among(cpu.status));
    assert!(StatusFlag::Negative.among(cpu.status));
    assert!(StatusFlag::Negative.among(cpu.status));

    cpu.load_and_run(vec![LDA_IMMEDIATE, 0xc0, STA_ZEROPAGE, 0x00, LDA_IMMEDIATE, 0x83, 0x24 /* bit */, 0x00, 0x00]);
    assert!(!StatusFlag::Zero.among(cpu.status));
    assert!(StatusFlag::Negative.among(cpu.status));
    assert!(StatusFlag::Negative.among(cpu.status));
}

#[test]
fn test_cmp() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![LDA_IMMEDIATE, 2, 0xc9, 1, 0x00]);
    assert!(StatusFlag::Carry.among(cpu.status));
    assert!(!StatusFlag::Zero.among(cpu.status));

    cpu.load_and_run(vec![LDA_IMMEDIATE, 2, 0xc9, 2, 0x00]);
    assert!(StatusFlag::Carry.among(cpu.status));
    assert!(StatusFlag::Zero.among(cpu.status));

    cpu.load_and_run(vec![LDA_IMMEDIATE, 1, 0xc9, 2, 0x00]);
    assert!(!StatusFlag::Carry.among(cpu.status));
    assert!(!StatusFlag::Zero.among(cpu.status));
}

#[test]
fn test_cpx() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![LDX_IMMEDIATE, 2, 0xe0, 1, 0x00]);
    assert!(StatusFlag::Carry.among(cpu.status));
    assert!(!StatusFlag::Zero.among(cpu.status));

    cpu.load_and_run(vec![LDX_IMMEDIATE, 2, 0xe0, 2, 0x00]);
    assert!(StatusFlag::Carry.among(cpu.status));
    assert!(StatusFlag::Zero.among(cpu.status));

    cpu.load_and_run(vec![LDX_IMMEDIATE, 1, 0xe0, 2, 0x00]);
    assert!(!StatusFlag::Carry.among(cpu.status));
    assert!(!StatusFlag::Zero.among(cpu.status));
}

#[test]
fn test_cpy() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![LDY_IMMEDIATE, 2, 0xc0, 1, 0x00]);
    assert!(StatusFlag::Carry.among(cpu.status));
    assert!(!StatusFlag::Zero.among(cpu.status));

    cpu.load_and_run(vec![LDY_IMMEDIATE, 2, 0xc0, 2, 0x00]);
    assert!(StatusFlag::Carry.among(cpu.status));
    assert!(StatusFlag::Zero.among(cpu.status));

    cpu.load_and_run(vec![LDY_IMMEDIATE, 1, 0xc0, 2, 0x00]);
    assert!(!StatusFlag::Carry.among(cpu.status));
    assert!(!StatusFlag::Zero.among(cpu.status));
}

#[test]
fn test_dec() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![LDA_IMMEDIATE, 0x02, STA_ZEROPAGE, 0x00, 0xc6 /* dec */, 0x00, LDA_ZEROPAGE, 0x0, 0x00]);
    assert_eq!(cpu.register_a, 0x1);
    assert!(!StatusFlag::Negative.among(cpu.status));
    assert!(!StatusFlag::Zero.among(cpu.status));

    cpu.load_and_run(vec![LDA_IMMEDIATE, 0x01, STA_ZEROPAGE, 0x00, 0xc6 /* dec */, 0x00, LDA_ZEROPAGE, 0x0, 0x00]);
    assert_eq!(cpu.register_a, 0x0);
    assert!(!StatusFlag::Negative.among(cpu.status));
    assert!(StatusFlag::Zero.among(cpu.status));

    cpu.load_and_run(vec![LDA_IMMEDIATE, 0x00, STA_ZEROPAGE, 0x00, 0xc6 /* dec */, 0x00, LDA_ZEROPAGE, 0x0, 0x00]);
    assert_eq!(cpu.register_a, 0xff);
    assert!(StatusFlag::Negative.among(cpu.status));
    assert!(!StatusFlag::Zero.among(cpu.status));
}

#[test]
fn test_inc() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![LDA_IMMEDIATE, 0xfe, STA_ZEROPAGE, 0x00, 0xe6 /* inc */, 0x00, LDA_ZEROPAGE, 0x0, 0x00]);
    assert_eq!(cpu.register_a, 0xff);
    assert!(StatusFlag::Negative.among(cpu.status));
    assert!(!StatusFlag::Zero.among(cpu.status));

    cpu.load_and_run(vec![LDA_IMMEDIATE, 0xff, STA_ZEROPAGE, 0x00, 0xe6 /* inc */, 0x00, LDA_ZEROPAGE, 0x0, 0x00]);
    assert_eq!(cpu.register_a, 0x0);
    assert!(!StatusFlag::Negative.among(cpu.status));
    assert!(StatusFlag::Zero.among(cpu.status));

    cpu.load_and_run(vec![LDA_IMMEDIATE, 0x0, STA_ZEROPAGE, 0x00, 0xe6 /* inc */, 0x00, LDA_ZEROPAGE, 0x0, 0x00]);
    assert_eq!(cpu.register_a, 0x1);
    assert!(!StatusFlag::Negative.among(cpu.status));
    assert!(!StatusFlag::Zero.among(cpu.status));
}

#[test]
fn test_jmp() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0x4c, 0x03, 0x80, 0x00]);
    
    // todo: indirect
}

#[test]
fn test_bcc() {
    // todo:
}
