// src/compression.rs

/// Bit-level writer for encoding compressed data.
pub struct BitWriter {
    buffer: Vec<u8>,
    current_byte: u8,
    bit_pos: u8, // 0..7, number of bits written in current_byte
}

impl BitWriter {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            current_byte: 0,
            bit_pos: 0,
        }
    }

    /// Write `n` bits from `value` (MSB first).
    pub fn write_bits(&mut self, value: u64, mut n: u8) {
        while n > 0 {
            let remaining = 8 - self.bit_pos;
            let bits_to_write = remaining.min(n);
            let shift = n - bits_to_write;
            let bits = ((value >> shift) & ((1u64 << bits_to_write) - 1)) as u8;
            self.current_byte |= bits << (remaining - bits_to_write);
            self.bit_pos += bits_to_write;
            n -= bits_to_write;
            if self.bit_pos == 8 {
                self.buffer.push(self.current_byte);
                self.current_byte = 0;
                self.bit_pos = 0;
            }
        }
    }

    /// Write a single bit.
    pub fn write_bit(&mut self, bit: bool) {
        self.write_bits(if bit { 1 } else { 0 }, 1);
    }

    /// Finalize and return the written bytes.
    pub fn finish(mut self) -> Vec<u8> {
        if self.bit_pos > 0 {
            self.buffer.push(self.current_byte);
        }
        self.buffer
    }
}

/// Bit-level reader for decoding compressed data.
pub struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u8, // 0..7, next bit to read in current byte
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_pos: 0,
            bit_pos: 0,
        }
    }

    /// Read `n` bits and return as u64 (MSB first).
    pub fn read_bits(&mut self, mut n: u8) -> Option<u64> {
        let mut result: u64 = 0;
        while n > 0 {
            if self.byte_pos >= self.data.len() {
                return None;
            }
            let remaining = 8 - self.bit_pos;
            let bits_to_read = remaining.min(n);
            let shift = remaining - bits_to_read;
            let mask = (((1u32 << bits_to_read) - 1) << shift) as u8;
            let bits = (self.data[self.byte_pos] & mask) >> shift;
            result = (result << bits_to_read) | (bits as u64);
            self.bit_pos += bits_to_read;
            n -= bits_to_read;
            if self.bit_pos == 8 {
                self.byte_pos += 1;
                self.bit_pos = 0;
            }
        }
        Some(result)
    }

    /// Read a single bit.
    pub fn read_bit(&mut self) -> Option<bool> {
        self.read_bits(1).map(|v| v == 1)
    }

    /// Check if there are more bits to read.
    pub fn has_remaining(&self) -> bool {
        self.byte_pos < self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_read_roundtrip_single_bits() {
        let mut writer = BitWriter::new();
        for i in 0..16 {
            writer.write_bit(i % 2 == 0);
        }
        let data = writer.finish();

        let mut reader = BitReader::new(&data);
        for i in 0..16 {
            assert_eq!(reader.read_bit(), Some(i % 2 == 0));
        }
    }

    #[test]
    fn test_write_read_roundtrip_multi_bit() {
        let mut writer = BitWriter::new();
        writer.write_bits(0b1010, 4);
        writer.write_bits(0b11001100, 8);
        writer.write_bits(0b111, 3);
        let data = writer.finish();

        let mut reader = BitReader::new(&data);
        assert_eq!(reader.read_bits(4), Some(0b1010));
        assert_eq!(reader.read_bits(8), Some(0b11001100));
        assert_eq!(reader.read_bits(3), Some(0b111));
    }

    #[test]
    fn test_write_read_large_value() {
        let mut writer = BitWriter::new();
        writer.write_bits(0xDEADBEEF, 32);
        let data = writer.finish();

        let mut reader = BitReader::new(&data);
        assert_eq!(reader.read_bits(32), Some(0xDEADBEEF));
    }

    #[test]
    fn test_empty_writer() {
        let writer = BitWriter::new();
        assert!(writer.finish().is_empty());
    }

    #[test]
    fn test_reader_past_end_returns_none() {
        let mut writer = BitWriter::new();
        writer.write_bits(0xFF, 8);
        let data = writer.finish();

        let mut reader = BitReader::new(&data);
        assert_eq!(reader.read_bits(8), Some(0xFF));
        assert_eq!(reader.read_bits(1), None);
    }
}
