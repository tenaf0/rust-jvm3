use std::io::Read;
use crate::class_parser::ParseError;

pub trait BEReader<const N: usize> : Sized {
    fn from_byte_array(buf: [u8; N]) -> Self;

    fn read(reader: &mut impl Read) -> Result<Self, ParseError> {
        let mut buf = [0u8; N];
        reader.read_exact(&mut buf)?;

        Ok(Self::from_byte_array(buf))
    }
}

// We can't abstract over from_be_bytes, because no specific trait implements it, so we have to resort to macros
macro_rules! be_reader {
    ($t: ty, $n: expr) => {
        impl BEReader<$n> for $t {
            fn from_byte_array(buf: [u8; $n]) -> Self {
                <$t>::from_be_bytes(buf)
            }
        }
    }
}

be_reader!(u8, 1);
be_reader!(u16, 2);
be_reader!(u32, 4);