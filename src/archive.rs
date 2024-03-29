use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

use crate::CfxResult;

pub trait FArchive {
    fn read_bytes(&mut self, buffer: &mut [u8]) -> CfxResult<usize>;
    fn set_position(&mut self, pos: u64) -> CfxResult<()>;
}

pub trait FArchiveExt: FArchive {
    fn read_uint(&mut self) -> CfxResult<u32>;
    fn read_int(&mut self) -> CfxResult<i32>;
}

impl<Archive> FArchiveExt for Archive
where
    Archive: FArchive,
{
    fn read_uint(&mut self) -> CfxResult<u32> {
        let mut buffer = [0u8; 4];
        self.read_bytes(&mut buffer)?;

        let mut reader = Cursor::new(buffer);
        let result = reader.read_u32::<LittleEndian>()?;

        Ok(result)
    }

    fn read_int(&mut self) -> CfxResult<i32> {
        let mut buffer = [0u8; 4];
        self.read_bytes(&mut buffer)?;

        let mut reader = Cursor::new(buffer);
        let result = reader.read_i32::<LittleEndian>()?;

        Ok(result)
    }
}

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
}

impl<Data> FArchive for FMemoryArchive<Data>
where
    Data: AsRef<[u8]>,
{
    fn read_bytes(&mut self, buffer: &mut [u8]) -> CfxResult<usize> {
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

    fn set_position(&mut self, pos: u64) -> CfxResult<()> {
        self.cursor.set_position(pos);
        Ok(())
    }
}

const VIRTUAL_BASE: u64 = 0x50000000;
const PHYSICAL_BASE: u64 = 0x60000000;

pub struct FResourceArchive<Data>
where
    Data: AsRef<[u8]>,
{
    virtual_stream: Cursor<Data>,
    physical_stream: Cursor<Data>,
    pos: u64,
}

impl<Data> FResourceArchive<Data>
where
    Data: AsRef<[u8]>,
{
    pub fn new(virtual_data: Data, physical_data: Data) -> Self {
        Self {
            virtual_stream: Cursor::new(virtual_data),
            physical_stream: Cursor::new(physical_data),
            pos: 0,
        }
    }

    pub fn read_ulong(&mut self) -> CfxResult<u64> {
        let mut buffer = [0u8; 8];
        self.read_bytes(&mut buffer)?;

        Ok(u64::from_le_bytes(buffer))
    }
}

impl<Data> FArchive for FResourceArchive<Data>
where
    Data: AsRef<[u8]>,
{
    fn read_bytes(&mut self, buffer: &mut [u8]) -> CfxResult<usize> {
        let mut base_position = 0x0;
        let mut cursor = if (self.pos & VIRTUAL_BASE) == VIRTUAL_BASE {
            base_position = VIRTUAL_BASE;
            &mut self.virtual_stream
        } else if (self.pos & PHYSICAL_BASE) == PHYSICAL_BASE {
            base_position = PHYSICAL_BASE;
            &mut self.physical_stream
        } else {
            return Err(format!("Invalid position: {}", self.pos).into());
        };

        cursor.set_position((self.pos & !base_position));
        let read = cursor.read(buffer)?;
        self.pos = self.pos | base_position;

        Ok(read)
    }

    fn set_position(&mut self, pos: u64) -> CfxResult<()> {
        self.pos = pos;
        Ok(())
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
