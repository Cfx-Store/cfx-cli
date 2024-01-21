use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

use crate::CfxResult;

pub struct FMemoryArchive<Data>
where
    Data: AsRef<[u8]>,
{
    pub len: usize,
    cursor: Cursor<Data>,
}

impl<Data> FMemoryArchive<Data>
where
    Data: AsRef<[u8]>,
{
    pub fn new(data: Data) -> Self {
        let len = data.as_ref().len();
        let cursor = Cursor::new(data);

        Self { len, cursor }
    }

    pub fn read_bytes(&mut self, buffer: &mut [u8]) -> CfxResult<usize> {
        let buffer_len = buffer.len();
        let total_len = self.cursor.position() as usize + buffer_len;
        if total_len > self.len {
            return Err(format!(
                "tried to read {buffer_len} bytes but there were only {total_len} bytes left"
            )
            .into());
        }

        let read = self.cursor.read(buffer)?;
        Ok(read)
    }

    pub fn read_uint(&mut self) -> CfxResult<u32> {
        Ok(self.cursor.read_u32::<LittleEndian>()?)
    }

    pub fn read_int(&mut self) -> CfxResult<i32> {
        Ok(self.cursor.read_i32::<LittleEndian>()?)
    }

    pub fn read_float(&mut self) -> CfxResult<f32> {
        Ok(self.cursor.read_f32::<LittleEndian>()?)
    }

    pub fn read_bool(&mut self) -> CfxResult<bool> {
        Ok(self.cursor.read_u8()? != 0)
    }

    pub fn read_string(&mut self) -> CfxResult<String> {
        let len = self.read_uint()? as usize;
        let mut buffer = vec![0u8; len];

        self.read_bytes(&mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
}

#[cfg(test)]
mod archive_tests {
    use super::*;

    #[test]
    fn archive_len_test() {
        let expected_len = 6;
        let data = vec![0u8; expected_len];
        let archive = FMemoryArchive::new(data);

        assert_eq!(archive.len, expected_len)
    }

    #[test]
    fn archive_read_bytes_test() {
        let expected_data: Vec<u8> = vec![1, 2, 3, 4, 5];
        let mut archive = FMemoryArchive::new(&expected_data);

        let mut buffer: [u8; 5] = Default::default();
        let result = archive.read_bytes(&mut buffer);

        assert!(result.is_ok(), "read_bytes returned an error");
        assert_eq!(result.unwrap(), expected_data.len());
        assert_eq!(buffer, expected_data.as_slice())
    }

    #[test]
    fn archive_read_bytes_overflow_test() {
        let mut archive = FMemoryArchive::new([69u8, 0, 0, 0]);
        let mut buffer: [u8; 5] = Default::default();
        let result = archive.read_bytes(&mut buffer);

        assert!(result.is_err(), "read_bytes did not return an error");
    }
}
