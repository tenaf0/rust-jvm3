use num_enum::{FromPrimitive};

pub enum InstructionResult {
    Continue,
    Return,
    Exception
}

#[derive(FromPrimitive, Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum Instruction {
    nop = 0,
    aconst_null = 1,
    iconst_m1 = 2,
    iconst_0 = 3,
    iconst_1 = 4,
    iconst_2 = 5,
    iconst_3 = 6,
    iconst_4 = 7,
    iconst_5 = 8,
    lconst_0 = 9,
    lconst_1 = 10,
    fconst_0 = 11,
    fconst_1 = 12,
    fconst_2 = 13,
    dconst_0 = 14,
    dconst_1 = 15,
    bipush = 16,
    sipush = 17,
    ldc = 18,
    ldc2_w = 20,
    iload = 21,
    fload = 23,
    dload = 24,
    aload = 25,
    iload_0 = 26,
    iload_1 = 27,
    iload_2 = 28,
    iload_3 = 29,
    lload_0 = 30,
    lload_1 = 31,
    lload_2 = 32,
    lload_3 = 33,
    dload_0 = 38,
    dload_1 = 39,
    dload_2 = 40,
    dload_3 = 41,
    aload_0 = 42,
    aload_1 = 43,
    aload_2 = 44,
    aload_3 = 45,
    iaload = 46,
    aaload = 50,
    istore = 54,
    dstore = 57,
    astore = 58,
    istore_0 = 59,
    istore_1 = 60,
    istore_2 = 61,
    istore_3 = 62,
    lstore_0 = 63,
    lstore_1 = 64,
    lstore_2 = 65,
    dstore_0 = 71,
    dstore_1 = 72,
    dstore_2 = 73,
    dstore_3 = 74,
    astore_0 = 75,
    astore_1 = 76,
    astore_2 = 77,
    astore_3 = 78,
    iastore = 79,
    aastore = 83,
    pop = 87,
    dup = 89,
    iadd = 96,
    ladd = 97,
    dadd = 99,
    isub = 100,
    dsub = 103,
    imul = 104,
    dmul = 107,
    ddiv = 111,
    dneg = 119,
    iinc = 132,
    i2l = 133,
    l2i = 136,
    l2d = 138,
    f2d = 141,
    ifne = 154,
    if_icmpge = 162,
    if_icmpgt = 163,
    goto = 167,
    ireturn = 172,
    lreturn = 173,
    dreturn = 175,
    areturn = 176,
    _return = 177,
    getstatic = 178,
    putstatic = 179,
    getfield = 180,
    putfield = 181,
    invokevirtual = 182,
    invokespecial = 183,
    invokestatic = 184,
    new = 187,
    newarray = 188,
    anewarray = 189,
    arraylength = 190,
    athrow = 191,
    breakpoint = 202,
    impdep1 = 254,
    #[default]
    impdep2 = 255
}

#[inline]
pub const fn instruction_length(instr: Instruction) -> usize {
    use Instruction::*;

    match instr {
        nop => 1,
        aconst_null => 1,
        iconst_m1 => 1,
        iconst_0 => 1,
        iconst_1 => 1,
        iconst_2 => 1,
        iconst_3 => 1,
        iconst_4 => 1,
        iconst_5 => 1,
        lconst_0 => 1,
        lconst_1 => 1,
        fconst_0 | fconst_1 | fconst_2 => 1,
        dconst_0 | dconst_1 => 1,
        bipush => 2,
        sipush => 3,
        ldc => 2,
        ldc2_w => 3,
        iload => 2,
        fload => 2,
        dload => 2,
        aload => 2,
        iload_0 => 1,
        iload_1 => 1,
        iload_2 => 1,
        iload_3 => 1,
        lload_0 => 1,
        lload_1 => 1,
        lload_2 => 1,
        lload_3 => 1,
        dload_0 | dload_1 | dload_2 | dload_3 => 1,
        aload_0 | aload_1 | aload_2 | aload_3 => 1,
        iaload => 1,
        aaload => 1,
        istore => 2,
        dstore => 2,
        astore => 2,
        istore_0 => 1,
        istore_1 => 1,
        istore_2 => 1,
        istore_3 => 1,
        lstore_0 => 1,
        lstore_1 => 1,
        lstore_2 => 1,
        dstore_0 | dstore_1 | dstore_2 | dstore_3 => 1,
        astore_0 | astore_1 | astore_2 | astore_3 => 1,
        iastore => 1,
        aastore => 1,
        pop => 1,
        dup => 1,
        iadd => 1,
        ladd => 1,
        dadd => 1,
        isub => 1,
        dsub => 1,
        imul => 1,
        dmul => 1,
        ddiv => 1,
        dneg => 1,
        iinc => 3,
        i2l => 1,
        l2i => 1,
        l2d => 1,
        f2d => 1,
        ifne | if_icmpge | if_icmpgt => 3,
        goto => 3,
        ireturn => 1,
        lreturn => 1,
        dreturn => 1,
        areturn => 1,
        _return => 1,
        getstatic => 3,
        putstatic => 3,
        getfield => 3,
        putfield => 3,
        invokevirtual => 3,
        invokespecial => 3,
        invokestatic => 3,
        new => 3,
        newarray => 2,
        anewarray => 3,
        arraylength => 1,
        athrow => 1,
        breakpoint => 1,
        impdep1 => 1,
        impdep2 => 1
    }
}

/*
#[inline]
pub fn execute_roots_only(frame: &mut Frame, code: &[u8]) {
    use Instruction::*;

    let instr = code[0];
    let instr = Instruction::try_from(instr).unwrap();
    match instr {
        aconst_null => frame.push(1),
        iconst_m1 | iconst_0 | iconst_1 | iconst_2 | iconst_3 | iconst_4 | iconst_5 => {
            frame.push(0);
        }
        lconst_0 | lconst_1 => {
            frame.push(0);
        }
        bipush | sipush => frame.push(0),
        ldc => frame.push(0), // TODO: It could be a String reference as well
        ldc2_w => frame.push(0),
        iload_0 | iload_1 | iload_2 | iload_3 => frame.push(0),
        lload_0 | lload_1 | lload_2 => frame.push(0),
        aload_0 | aload_1 | aload_2 | aload_3 => todo!(),
        istore => {}
        istore_0 => {}
        istore_1 => {}
        istore_2 => {}
        istore_3 => {}
        lstore_0 => {}
        lstore_1 => {}
        lstore_2 => {}
        astore_0 | astore_1 | astore_2 | astore_3 => {
            // TODO
        }
        dup => {
            frame.push(frame.safe_peek().unwrap());
        }
        iadd => {}
        ladd => {}
        isub => { frame.pop(); frame.pop(); }
        imul => { frame.pop(); },
        iinc => {}
        i2l => {}
        l2i => {}
        if_icmpge | if_icmpgt => { frame.pop(); frame.pop(); }
        goto => {}
        lreturn => {}
        _return => {}
        getstatic => {}, // TODO: Depends on field type
        putstatic => { frame.pop(); }
        getfield => {} // TODO: Depends on method
        putfield => {
            frame.pop();
            frame.pop();
        } // TODO: Depends on method
        invokespecial => {} // TODO: Depends on method
        new => frame.push(1)
    }
}*/