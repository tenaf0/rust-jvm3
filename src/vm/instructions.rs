use num_enum::TryFromPrimitive;
use crate::vm::thread::frame::Frame;

pub enum InstructionResult {
    Continue,
    Return
}

#[derive(TryFromPrimitive, Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum Instruction {
    iconst_m1 = 2,
    iconst_0 = 3,
    iconst_1 = 4,
    iconst_2 = 5,
    iconst_3 = 6,
    iconst_4 = 7,
    iconst_5 = 8,
    bipush = 16,
    sipush = 17,
    ldc = 18,
    ldc2_w = 20,
    iload_0 = 26,
    iload_1 = 27,
    iload_2 = 28,
    iload_3 = 29,
    lload_0 = 30,
    lload_1 = 31,
    lload_2 = 32,
    istore = 54,
    istore_0 = 59,
    istore_1 = 60,
    istore_2 = 61,
    istore_3 = 62,
    lstore_0 = 63,
    lstore_1 = 64,
    lstore_2 = 65,
    iadd = 96,
    ladd = 97,
    imul = 104,
    iinc = 132,
    i2l = 133,
    if_icmpge = 162,
    if_icmpgt = 163,
    goto = 167,
    lreturn = 173,
    _return = 177,
    getstatic = 178,
    putstatic = 179,
}

pub const fn instruction_length(instr: Instruction) -> usize {
    use Instruction::*;

    match instr {
        iconst_m1 => 1,
        iconst_0 => 1,
        iconst_1 => 1,
        iconst_2 => 1,
        iconst_3 => 1,
        iconst_4 => 1,
        iconst_5 => 1,
        bipush => 2,
        sipush => 3,
        ldc => 2,
        ldc2_w => 3,
        iload_0 => 1,
        iload_1 => 1,
        iload_2 => 1,
        iload_3 => 1,
        lload_0 => 2,
        lload_1 => 2,
        lload_2 => 2,
        istore => 2,
        istore_0 => 1,
        istore_1 => 1,
        istore_2 => 1,
        istore_3 => 1,
        lstore_0 => 1,
        lstore_1 => 1,
        lstore_2 => 1,
        iadd => 1,
        ladd => 1,
        imul => 1,
        iinc => 3,
        i2l => 1,
        if_icmpge | if_icmpgt => 3,
        goto => 3,
        lreturn => 1,
        _return => 1,
        getstatic => 3,
        putstatic => 3
    }
}

#[inline]
pub fn execute_roots_only(frame: &mut Frame, code: &[u8]) {
    use Instruction::*;

    let instr = code[0];
    let instr = Instruction::try_from(instr).unwrap();
    match instr {
        iconst_m1 | iconst_0 | iconst_1 | iconst_2 | iconst_3 | iconst_4 | iconst_5 => {
            frame.push(0);
        }
        bipush | sipush => frame.push(0),
        ldc => frame.push(0), // TODO: It could be a String reference as well
        ldc2_w => frame.push(0),
        iload_0 | iload_1 | iload_2 | iload_3 => frame.push(0),
        lload_0 | lload_1 | lload_2 => frame.push(0),
        istore => {}
        istore_0 => {}
        istore_1 => {}
        istore_2 => {}
        istore_3 => {}
        lstore_0 => {}
        lstore_1 => {}
        lstore_2 => {}
        iadd => {}
        ladd => {}
        imul => { frame.pop(); },
        iinc => {}
        i2l => {}
        if_icmpge | if_icmpgt => { frame.pop(); frame.pop(); }
        goto => {}
        lreturn => {}
        _return => {}
        getstatic => {}, // TODO: Depends on field type
        putstatic => { frame.pop(); }
    }
}