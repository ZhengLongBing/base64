use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum Base64Error {
    // 基本错误
    InvalidLength,
    InvalidCharacter,

    // 缓冲区错误
    BufferOverflow,
    BufferUnderflow,
}

impl Error for Base64Error {}

impl fmt::Display for Base64Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength => write!(f, "Invalid input length"),
            Self::InvalidCharacter => write!(f, "Invalid character in input"),
            Self::BufferOverflow => write!(f, "Buffer overflow"),
            Self::BufferUnderflow => write!(f, "Buffer underflow"),
        }
    }
}
