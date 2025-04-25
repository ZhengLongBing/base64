# Base64 流式编解码库

这是一个用Rust实现的高性能Base64编解码库，支持流式处理和多种编码模式。

## 特性

- 支持标准Base64和URL安全Base64编码
- 支持流式处理，适合处理大文件
- 支持可选的填充字符
- 使用循环缓冲区实现高效的内存使用
- 线程安全

## 使用方法

### 基本编码示例

```rust
use base64::{Base64StreamEncoder, Base64Mode};

// 标准Base64编码
let mut encoder = Base64StreamEncoder::standard();
encoder.write_exact(b"Hello, World!").unwrap();
encoder.finalize().unwrap();

let mut output = vec![0; 20];
let n = encoder.read(&mut output);
assert_eq!(
    String::from_utf8(output[..n].to_vec()).unwrap(),
    "SGVsbG8sIFdvcmxkIQ=="
);

// URL安全Base64编码（无填充）
let mut encoder = Base64StreamEncoder::new(Base64Mode::UrlSafe, false);
encoder.write_exact(b"?&=").unwrap();
encoder.finalize().unwrap();

let mut output = vec![0; 20];
let n = encoder.read(&mut output);
// 输出: PyY9
```

### 基本解码示例

```rust
use base64::{Base64StreamDecoder, Base64Mode};

// 标准Base64解码
let mut decoder = Base64StreamDecoder::standard();
decoder.write_exact(b"SGVsbG8sIFdvcmxkIQ==").unwrap();
decoder.finalize().unwrap();

let mut output = vec![0; 20];
let n = decoder.read(&mut output);
assert_eq!(
    String::from_utf8(output[..n].to_vec()).unwrap(),
    "Hello, World!"
);

// URL安全Base64解码（无填充要求）
let mut decoder = Base64StreamDecoder::new(Base64Mode::UrlSafe, false);
decoder.write_exact(b"PyY9").unwrap();
decoder.finalize().unwrap();
```

### 流式处理示例

编码：
```rust
let mut encoder = Base64StreamEncoder::standard();

// 分块写入数据
encoder.write_exact(b"Hel").unwrap();
encoder.write_exact(b"lo, ").unwrap();
encoder.write_exact(b"World!").unwrap();
encoder.finalize().unwrap();

let mut output = vec![0; 20];
let n = encoder.read(&mut output);
// 输出: SGVsbG8sIFdvcmxkIQ==
```

解码：
```rust
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
// 输出: Hello, World!
```

## API 说明

### Base64StreamEncoder

- `standard()` - 创建标准Base64编码器（带填充）
- `new(mode: Base64Mode, padding: bool)` - 创建自定义编码器
- `write(&mut self, data: &[u8]) -> usize` - 写入数据
- `write_exact(&mut self, data: &[u8]) -> Result<(), Base64Error>` - 精确写入数据
- `read(&mut self, data: &mut [u8]) -> usize` - 读取编码后的数据
- `read_exact(&mut self, data: &mut [u8]) -> Result<(), Base64Error>` - 精确读取数据
- `finalize(&mut self) -> Result<(), Base64Error>` - 完成编码

### Base64StreamDecoder

- `standard()` - 创建标准Base64解码器（需要填充）
- `new(mode: Base64Mode, require_padding: bool)` - 创建自定义解码器
- `write(&mut self, data: &[u8]) -> usize` - 写入编码数据
- `write_exact(&mut self, data: &[u8]) -> Result<(), Base64Error>` - 精确写入编码数据
- `read(&mut self, data: &mut [u8]) -> usize` - 读取解码后的数据
- `read_exact(&mut self, data: &mut [u8]) -> Result<(), Base64Error>` - 精确读取数据
- `finalize(&mut self) -> Result<(), Base64Error>` - 完成解码

## 错误处理

库使用 `Base64Error` 枚举处理各种错误情况：
- 无效字符
- 无效长度
- 缓冲区已满
- 数据不足

## 性能考虑

- 使用固定大小的循环缓冲区，避免频繁的内存分配
- 编码缓冲区大小：4KB
- 解码缓冲区大小：3KB
- 支持分块处理，适合处理大文件