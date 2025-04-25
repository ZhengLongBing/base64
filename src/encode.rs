use crate::circular_buffer::CircularBuffer;
use crate::error::Base64Error;
use crate::mode::{Base64Mode, STANDARD_ENCODE_TABLE, URL_SAFE_ENCODE_TABLE};

const INPUT_BUFFER_SIZE: usize = 3 * 1024; // 3KB 块大小（能被3整除）
const OUTPUT_BUFFER_SIZE: usize = 4 * 1024; // 4KB 输出块大小（能被4整除）

const CHUNK_SIZE: usize = 3;

pub struct Base64StreamEncoder {
    padding: bool,
    input_buffer: CircularBuffer<INPUT_BUFFER_SIZE>,
    output_buffer: CircularBuffer<OUTPUT_BUFFER_SIZE>,
    encode_table: &'static [u8],
}

impl Base64StreamEncoder {
    pub fn standard() -> Self {
        Self {
            padding: true,
            input_buffer: CircularBuffer::new(),
            output_buffer: CircularBuffer::new(),
            encode_table: STANDARD_ENCODE_TABLE,
        }
    }
    pub fn new(mode: Base64Mode, padding: bool) -> Self {
        let encode_table = match mode {
            Base64Mode::Standard => STANDARD_ENCODE_TABLE,
            Base64Mode::UrlSafe => URL_SAFE_ENCODE_TABLE,
        };
        Self {
            padding,
            input_buffer: CircularBuffer::new(),
            output_buffer: CircularBuffer::new(),
            encode_table,
        }
    }

    pub fn read(&mut self, data: &mut [u8]) -> usize {
        self.output_buffer.read(data)
    }

    pub fn write(&mut self, data: &[u8]) -> usize {
        self.input_buffer.write(data)
    }

    pub fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Base64Error> {
        self.output_buffer.read_exact(data)
    }

    pub fn write_exact(&mut self, data: &[u8]) -> Result<(), Base64Error> {
        self.input_buffer.write_exact(data)
    }

    // 处理输入数据
    pub fn process(&mut self) -> Result<(), Base64Error> {
        // 处理输入数据的完整块
        let remaining = self.input_buffer.available_data();
        let chunks = remaining / CHUNK_SIZE;

        let mut chunk_buffer = [0_u8; CHUNK_SIZE];
        for _ in 0..chunks {
            self.input_buffer.read_exact(&mut chunk_buffer)?;
            self.encode_chunk(&chunk_buffer)?;
        }
        Ok(())
    }

    // 完成编码，处理剩余数据
    pub fn finalize(&mut self) -> Result<(), Base64Error> {
        self.process()?;

        if self.input_buffer.available_data() == 0 {
            return Ok(());
        }

        let mut chunk_buffer = [0_u8; CHUNK_SIZE];

        let n = self.input_buffer.read(&mut chunk_buffer);

        let b1 = chunk_buffer[0];
        let b2 = chunk_buffer[1];
        let b3 = chunk_buffer[2];

        let c1 = b1 >> 2;
        let c2 = ((b1 & 0b11) << 4) | (b2 >> 4);
        let c3 = ((b2 & 0b1111) << 2) | (b3 >> 6);
        let c4 = b3 & 0b111111;

        let c1 = self.encode_table[c1 as usize];
        let c2 = self.encode_table[c2 as usize];
        let c3 = self.encode_table[c3 as usize];
        let c4 = self.encode_table[c4 as usize];

        match n {
            1 => match self.padding {
                true => self.output_buffer.write_exact(&[c1, c2, b'=', b'=']),
                false => self.output_buffer.write_exact(&[c1, c2]),
            },
            2 => match self.padding {
                true => self.output_buffer.write_exact(&[c1, c2, c3, b'=']),
                false => self.output_buffer.write_exact(&[c1, c2, c3]),
            },
            3 => self.output_buffer.write_exact(&[c1, c2, c3, c4]),
            _ => unreachable!(),
        }
    }

    // 辅助函数：编码3字节块
    fn encode_chunk(&mut self, chunk: &[u8]) -> Result<(), Base64Error> {
        let b1 = chunk[0];
        let b2 = chunk[1];
        let b3 = chunk[2];

        let c1 = b1 >> 2;
        let c2 = ((b1 & 0b11) << 4) | (b2 >> 4);
        let c3 = ((b2 & 0b1111) << 2) | (b3 >> 6);
        let c4 = b3 & 0b111111;

        let c1 = self.encode_table[c1 as usize];
        let c2 = self.encode_table[c2 as usize];
        let c3 = self.encode_table[c3 as usize];
        let c4 = self.encode_table[c4 as usize];

        let ref chunk = [c1, c2, c3, c4];
        self.output_buffer.write_exact(chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::Base64StreamEncoder;
    use crate::mode::Base64Mode;

    // 辅助函数：编码整个字符串
    fn encode_string(input: &str, mode: Base64Mode, padding: bool) -> Result<String, Base64Error> {
        let mut encoder = Base64StreamEncoder::new(mode, padding);
        encoder.write_exact(input.as_bytes())?;
        encoder.finalize()?;

        let mut output = vec![0; encoder.output_buffer.available_data()];
        encoder.read_exact(&mut output)?;

        Ok(String::from_utf8(output).unwrap())
    }

    #[test]
    fn test_standard_base64_encode() {
        assert_eq!(
            encode_string("Hello, World!", Base64Mode::Standard, true).unwrap(),
            "SGVsbG8sIFdvcmxkIQ=="
        );
        assert_eq!(encode_string("", Base64Mode::Standard, true).unwrap(), "");
        assert_eq!(
            encode_string("f", Base64Mode::Standard, true).unwrap(),
            "Zg=="
        );
        assert_eq!(
            encode_string("fo", Base64Mode::Standard, true).unwrap(),
            "Zm8="
        );
        assert_eq!(
            encode_string("foo", Base64Mode::Standard, true).unwrap(),
            "Zm9v"
        );
    }

    #[test]
    fn test_url_safe_base64_encode() {
        assert_eq!(
            encode_string("Hello, World!", Base64Mode::UrlSafe, true).unwrap(),
            "SGVsbG8sIFdvcmxkIQ=="
        );
        // 测试包含特殊字符的情况
        assert_eq!(
            encode_string("?&=", Base64Mode::UrlSafe, true).unwrap(),
            "PyY9"
        );
    }

    #[test]
    fn test_no_padding_encode() {
        assert_eq!(
            encode_string("f", Base64Mode::Standard, false).unwrap(),
            "Zg"
        );
        assert_eq!(
            encode_string("fo", Base64Mode::Standard, false).unwrap(),
            "Zm8"
        );
    }

    #[test]
    fn test_streaming_encode() {
        let mut encoder = Base64StreamEncoder::standard();

        // 分块写入数据
        encoder.write_exact(b"Hel").unwrap();
        encoder.write_exact(b"lo, ").unwrap();
        encoder.write_exact(b"World!").unwrap();
        encoder.finalize().unwrap();

        let mut output = vec![0; 20];
        let n = encoder.read(&mut output);
        assert_eq!(
            String::from_utf8(output[..n].to_vec()).unwrap(),
            "SGVsbG8sIFdvcmxkIQ=="
        );
    }
}
