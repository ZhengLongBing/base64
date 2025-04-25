#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Base64Mode {
    Standard,
    UrlSafe,
}

// 标准Base64字符表
pub const STANDARD_ENCODE_TABLE: &'static [u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

// URL安全的Base64字符表
pub const URL_SAFE_ENCODE_TABLE: &'static [u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

// 创建解码表
const fn create_decode_table(encode_table: &[u8]) -> [i8; 256] {
    let mut table = [-1i8; 256];
    let mut i = 0;
    while i < 64 {
        table[encode_table[i] as usize] = i as i8;
        i += 1;
    }
    table
}

// 标准Base64解码表
const __STANDARD_DECODE_TABLE: [i8; 256] = create_decode_table(STANDARD_ENCODE_TABLE);
pub const STANDARD_DECODE_TABLE: &'static [i8] = &__STANDARD_DECODE_TABLE;

// URL安全的Base64解码表
const __URL_SAFE_DECODE_TABLE: [i8; 256] = create_decode_table(URL_SAFE_ENCODE_TABLE);
pub const URL_SAFE_DECODE_TABLE: &'static [i8] = &__STANDARD_DECODE_TABLE;
