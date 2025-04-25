use crate::circular_buffer::CircularBuffer;
use crate::error::Base64Error;
use crate::mode::{Base64Mode, STANDARD_DECODE_TABLE, URL_SAFE_DECODE_TABLE};

const INPUT_BUFFER_SIZE: usize = 4 * 1024; // 4KB 输出块大小（能被4整除）
const OUTPUT_BUFFER_SIZE: usize = 3 * 1024; // 3KB 块大小（能被3整除）
const CHUNK_SIZE: usize = 4;
pub struct Base64StreamDecoder {
    require_padding: bool,
    input_buffer: CircularBuffer<INPUT_BUFFER_SIZE>,
    output_buffer: CircularBuffer<OUTPUT_BUFFER_SIZE>,
    decode_table: &'static [i8],
}

impl Base64StreamDecoder {
    pub fn standard() -> Self {
        Self {
            require_padding: true,
            input_buffer: CircularBuffer::new(),
            output_buffer: CircularBuffer::new(),
            decode_table: STANDARD_DECODE_TABLE,
        }
    }
    pub fn new(mode: Base64Mode, require_padding: bool) -> Self {
        let decode_table = match mode {
            Base64Mode::Standard => STANDARD_DECODE_TABLE,
            Base64Mode::UrlSafe => URL_SAFE_DECODE_TABLE,
        };
        Self {
            require_padding,
            input_buffer: CircularBuffer::new(),
            output_buffer: CircularBuffer::new(),
            decode_table,
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

    pub fn process(&mut self) -> Result<(), Base64Error> {
        // 处理完整的4字节块
        let remaining = self.input_buffer.available_data();
        let chunks = remaining / CHUNK_SIZE;

        let mut chunk_buffer = [0_u8; CHUNK_SIZE];
        for _ in 0..chunks {
            self.input_buffer.read(&mut chunk_buffer);
            self.decode_chunk(&chunk_buffer)?;
        }
        Ok(())
    }

    pub fn finalize(&mut self) -> Result<(), Base64Error> {
        if self.require_padding && self.input_buffer.available_data() % 4 != 0 {
            return Err(Base64Error::InvalidLength);
        }

        self.process()?;

        if self.input_buffer.available_data() == 0 {
            return Ok(());
        }

        // 如果不需要填充，添加虚拟的填充字符
        if !self.require_padding {
            while self.input_buffer.available_data() < 4 {
                self.input_buffer.write_exact(&[b'='])?;
            }
        }

        let mut chunk_buffer = [0_u8; CHUNK_SIZE];
        self.input_buffer.read_exact(&mut chunk_buffer)?;
        self.decode_chunk(&chunk_buffer)
    }
    fn decode_chunk(&mut self, chunk: &[u8]) -> Result<(), Base64Error> {
        let c1 = self.decode_table[chunk[0] as usize];
        let c2 = self.decode_table[chunk[1] as usize];

        if c1 < 0 || c2 < 0 {
            return Err(Base64Error::InvalidCharacter);
        }
        let c1 = c1 as u8;
        let c2 = c2 as u8;
        let b1 = (c1 << 2) | (c2 >> 4);

        self.output_buffer.write_exact(&[b1])?;

        if chunk[2] != b'=' {
            let c3 = self.decode_table[chunk[2] as usize];
            if c3 < 0 {
                return Err(Base64Error::InvalidCharacter);
            }
            let c3 = c3 as u8;
            let b2 = (c2 << 4) | (c3 >> 2);
            self.output_buffer.write_exact(&[b2])?;
            if chunk[3] != b'=' {
                let c4 = self.decode_table[chunk[3] as usize];
                if c4 < 0 {
                    return Err(Base64Error::InvalidCharacter);
                }
                let c4 = c4 as u8;
                let b3 = (c3 << 6) | c4;
                self.output_buffer.write_exact(&[b3])?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::Base64StreamDecoder;
    use crate::mode::Base64Mode;

    // 辅助函数：解码整个字符串
    fn decode_string(
        input: &str,
        mode: Base64Mode,
        require_padding: bool,
    ) -> Result<Vec<u8>, Base64Error> {
        let mut decoder = Base64StreamDecoder::new(mode, require_padding);
        decoder.write_exact(input.as_bytes())?;
        decoder.finalize()?;

        let mut output = vec![0; decoder.output_buffer.available_data()];
        decoder.read_exact(&mut output)?;

        Ok(output)
    }

    #[test]
    fn test_standard_base64_decode() {
        assert_eq!(
            String::from_utf8(
                decode_string("SGVsbG8sIFdvcmxkIQ==", Base64Mode::Standard, true).unwrap()
            )
            .unwrap(),
            "Hello, World!"
        );
        assert_eq!(
            decode_string("", Base64Mode::Standard, true).unwrap(),
            Vec::<u8>::new()
        );
    }

    #[test]
    fn test_url_safe_base64_decode() {
        assert_eq!(
            String::from_utf8(
                decode_string("SGVsbG8sIFdvcmxkIQ==", Base64Mode::UrlSafe, true).unwrap()
            )
            .unwrap(),
            "Hello, World!"
        );
    }

    #[test]
    fn test_no_padding_decode() {
        assert_eq!(
            String::from_utf8(decode_string("Zg", Base64Mode::Standard, false).unwrap()).unwrap(),
            "f"
        );
        assert_eq!(
            String::from_utf8(decode_string("Zm8", Base64Mode::Standard, false).unwrap()).unwrap(),
            "fo"
        );
    }

    #[test]
    fn test_streaming_decode() {
        let mut decoder = Base64StreamDecoder::standard();

        // 分块写入数据
        decoder.write_exact(b"SGVs").unwrap();
        decoder.write_exact(b"bG8s").unwrap();
        decoder.write_exact(b"IFdv").unwrap();
        decoder.write_exact(b"cmxk").unwrap();
        decoder.write_exact(b"IQ==").unwrap();
        decoder.finalize().unwrap();

        let mut output = vec![0; 20];
        let n = decoder.read(&mut output);
        assert_eq!(
            String::from_utf8(output[..n].to_vec()).unwrap(),
            "Hello, World!"
        );
    }
}
