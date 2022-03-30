use num_enum::TryFromPrimitive;

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum Instruction {
    bipush = 16,
    sipush = 17,
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
    lreturn = 173,
    _return = 177,
    getstatic = 178,
}