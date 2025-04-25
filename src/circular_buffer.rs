use crate::error::Base64Error;

/// 固定大小的循环缓冲区
pub struct CircularBuffer<const BUFFER_SIZE: usize> {
    buffer: Box<[u8; BUFFER_SIZE]>,
    read_pos: usize,
    write_pos: usize,
    size: usize,
}

impl<const BUFFER_SIZE: usize> CircularBuffer<BUFFER_SIZE> {
    pub fn new() -> Self {
        Self {
            buffer: Box::new([0; BUFFER_SIZE]),
            read_pos: 0,
            write_pos: 0,
            size: 0,
        }
    }

    pub fn available_space(&self) -> usize {
        BUFFER_SIZE - self.size
    }

    pub fn available_data(&self) -> usize {
        self.size
    }

    pub fn write(&mut self, data: &[u8]) -> usize {
        let write_len = data.len().min(self.available_space());
        let first_write = (BUFFER_SIZE - self.write_pos).min(write_len);

        // 写入第一部分
        self.buffer[self.write_pos..self.write_pos + first_write]
            .copy_from_slice(&data[..first_write]);

        // 如果需要，写入剩余部分
        if first_write < write_len {
            let second_write = write_len - first_write;
            self.buffer[..second_write].copy_from_slice(&data[first_write..write_len]);
        }

        self.write_pos = (self.write_pos + write_len) % BUFFER_SIZE;
        self.size += write_len;
        write_len
    }

    pub fn read(&mut self, data: &mut [u8]) -> usize {
        let read_len = data.len().min(self.size);
        let first_read = (BUFFER_SIZE - self.read_pos).min(read_len);

        // 读取第一部分
        data[..first_read].copy_from_slice(&self.buffer[self.read_pos..self.read_pos + first_read]);

        // 如果需要，读取剩余部分
        if first_read < read_len {
            let second_read = read_len - first_read;
            data[first_read..read_len].copy_from_slice(&self.buffer[..second_read]);
        }

        self.read_pos = (self.read_pos + read_len) % BUFFER_SIZE;
        self.size -= read_len;
        read_len
    }

    pub fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Base64Error> {
        if self.read(data) != data.len() {
            return Err(Base64Error::BufferUnderflow);
        }
        Ok(())
    }

    pub fn write_exact(&mut self, data: &[u8]) -> Result<(), Base64Error> {
        if self.write(data) != data.len() {
            return Err(Base64Error::BufferUnderflow);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试基本初始化
    #[test]
    fn test_new_buffer() {
        let buffer = CircularBuffer::<8>::new();
        assert_eq!(buffer.available_space(), 8);
        assert_eq!(buffer.available_data(), 0);
    }

    // 测试基本写入
    #[test]
    fn test_basic_write() {
        let mut buffer = CircularBuffer::<8>::new();
        let data = [1, 2, 3, 4];
        let written = buffer.write(&data);

        assert_eq!(written, 4);
        assert_eq!(buffer.available_space(), 4);
        assert_eq!(buffer.available_data(), 4);
    }

    // 测试基本读取
    #[test]
    fn test_basic_read() {
        let mut buffer = CircularBuffer::<8>::new();
        let write_data = [1, 2, 3, 4];
        buffer.write(&write_data);

        let mut read_data = [0; 4];
        let read = buffer.read(&mut read_data);

        assert_eq!(read, 4);
        assert_eq!(read_data, write_data);
        assert_eq!(buffer.available_space(), 8);
        assert_eq!(buffer.available_data(), 0);
    }

    // 测试缓冲区满时的写入
    #[test]
    fn test_write_full_buffer() {
        let mut buffer = CircularBuffer::<4>::new();
        let data = [1, 2, 3, 4, 5];
        let written = buffer.write(&data);

        assert_eq!(written, 4); // 只能写入4个字节
        assert_eq!(buffer.available_space(), 0);
    }

    // 测试循环写入
    #[test]
    fn test_circular_write() {
        let mut buffer = CircularBuffer::<4>::new();

        // 第一次写入
        buffer.write(&[1, 2]);

        // 读取一个字节
        let mut read_data = [0];
        buffer.read(&mut read_data);

        // 再次写入，此时应该循环到开头
        let written = buffer.write(&[3, 4]);
        assert_eq!(written, 2);

        let mut result = [0; 3];
        let read = buffer.read(&mut result);
        assert_eq!(read, 3);
        assert_eq!(result, [2, 3, 4]);
    }

    // 测试exact读写
    #[test]
    fn test_exact_operations() {
        let mut buffer = CircularBuffer::<4>::new();

        // 测试write_exact
        assert!(buffer.write_exact(&[1, 2]).is_ok());
        assert!(buffer.write_exact(&[3, 4, 5]).is_err()); // 空间不足

        // 测试read_exact
        let mut read_data = [0; 2];
        assert!(buffer.read_exact(&mut read_data).is_ok());
        assert_eq!(read_data, [1, 2]);

        let mut read_data = [0; 3];
        // 测试读取空缓冲区
        assert!(buffer.read_exact(&mut read_data).is_err());
    }

    // 测试大数据分段写入
    #[test]
    fn test_large_write() {
        let mut buffer = CircularBuffer::<8>::new();
        let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let written = buffer.write(&data);
        assert_eq!(written, 8); // 只能写入8个字节

        let mut read_data = [0; 8];
        buffer.read(&mut read_data);
        assert_eq!(read_data, [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    // 测试零长度操作
    #[test]
    fn test_zero_length_operations() {
        let mut buffer = CircularBuffer::<4>::new();

        assert_eq!(buffer.write(&[]), 0);

        let mut read_data = [];
        assert_eq!(buffer.read(&mut read_data), 0);
    }
}
