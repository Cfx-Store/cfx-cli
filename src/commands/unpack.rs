use deflate::deflate_bytes;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::archive::FMemoryArchive;
use crate::CfxResult;

const MAGIC: u32 = 0x37435352;

const BUCKETS_CAPACITY: [u32; 9] = [0x1, 0x3, 0xF, 0x3F, 0x7F, 0x1, 0x1, 0x1, 0x1];
const BUCKETS_SHIFTS: [usize; 9] = [4, 5, 7, 11, 17, 24, 25, 26, 27];

struct ResourceChunkFlags {
    value: u32,
    type_val: u32,
    base_shift: u32,
    base_size: u32,
}

impl ResourceChunkFlags {
    pub fn new(value: u32) -> Self {
        let base_shift = value & 0xF;
        Self {
            value,
            type_val: (value >> 28) & 0xF,
            base_shift,
            base_size: (0x200u32 << base_shift as i32),
        }
    }

    fn get_chunk_sizes(&self) -> Vec<u32> {
        let result: Vec<u32> = vec![
            self.base_size << 8,
            self.base_size << 7,
            self.base_size << 6,
            self.base_size << 5,
            self.base_size << 4,
            self.base_size << 3,
            self.base_size << 2,
            self.base_size << 1,
            self.base_size << 0,
        ];

        result
    }

    fn get_buckets_count(&self) -> Vec<u32> {
        let result: Vec<u32> = vec![
            (self.value >> BUCKETS_SHIFTS[0]) & BUCKETS_CAPACITY[0],
            (self.value >> BUCKETS_SHIFTS[1]) & BUCKETS_CAPACITY[1],
            (self.value >> BUCKETS_SHIFTS[2]) & BUCKETS_CAPACITY[2],
            (self.value >> BUCKETS_SHIFTS[3]) & BUCKETS_CAPACITY[3],
            (self.value >> BUCKETS_SHIFTS[4]) & BUCKETS_CAPACITY[4],
            (self.value >> BUCKETS_SHIFTS[5]) & BUCKETS_CAPACITY[5],
            (self.value >> BUCKETS_SHIFTS[6]) & BUCKETS_CAPACITY[6],
            (self.value >> BUCKETS_SHIFTS[7]) & BUCKETS_CAPACITY[7],
            (self.value >> BUCKETS_SHIFTS[8]) & BUCKETS_CAPACITY[8],
        ];

        result
    }

    fn get_buckets_sizes(&self) -> Vec<u32> {
        let chunk_sizes = self.get_chunk_sizes();
        let buckets_count = self.get_buckets_count();
        let result: Vec<u32> = vec![
            chunk_sizes[0] * buckets_count[0],
            chunk_sizes[1] * buckets_count[1],
            chunk_sizes[2] * buckets_count[2],
            chunk_sizes[3] * buckets_count[3],
            chunk_sizes[4] * buckets_count[4],
            chunk_sizes[5] * buckets_count[5],
            chunk_sizes[6] * buckets_count[6],
            chunk_sizes[7] * buckets_count[7],
            chunk_sizes[8] * buckets_count[8],
        ];

        result
    }

    fn get_size(&self) -> u32 {
        let buckets_sizes = self.get_buckets_sizes();
        return buckets_sizes[0]
            + buckets_sizes[1]
            + buckets_sizes[2]
            + buckets_sizes[3]
            + buckets_sizes[4]
            + buckets_sizes[5]
            + buckets_sizes[6]
            + buckets_sizes[7]
            + buckets_sizes[8];
    }
}

#[derive(Debug)]
struct ArchiveHeader {
    pub flags: u32,
    pub virtual_page_flags: u32,
    pub physical_page_flags: u32,
    pub version: i32,
}

impl ArchiveHeader {
    pub fn from<Data>(archive: &mut FMemoryArchive<Data>) -> CfxResult<Self>
    where
        Data: AsRef<[u8]>,
    {
        Ok(ArchiveHeader {
            flags: archive.read_uint()?,
            virtual_page_flags: archive.read_uint()?,
            physical_page_flags: archive.read_uint()?,
            version: archive.read_int()? & 0xFF,
        })
    }
}

pub fn handle_unpack_command(filename: &str) -> CfxResult<()> {
    let filepath = Path::new(filename);
    if !filepath.exists() || !filepath.is_file() {
        return Err("File does not exist".into());
    }

    let mut file = File::open(filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    log::info!("Loaded file ({} bytes)", buffer.len());

    let mut archive = FMemoryArchive::new(buffer);
    let magic = archive.read_uint()?;
    if magic != MAGIC {
        return Err(format!("Invalid magic: {:#04x} (expected: {:#04x})", magic, MAGIC).into());
    }

    let header = ArchiveHeader::from(&mut archive)?;
    log::info!("Header: {:?}", header);

    let virtual_flags = ResourceChunkFlags::new(header.virtual_page_flags);
    let physical_flags = ResourceChunkFlags::new(header.virtual_page_flags);
    log::info!("Virtual size: {:?}", virtual_flags.get_size());
    log::info!("Physical size: {:?}", physical_flags.get_size());

    let mut virtual_buffer = vec![0u8; virtual_flags.get_size() as usize];
    let mut physical_buffer = vec![0u8; physical_flags.get_size() as usize];
    archive.read_bytes(&mut virtual_buffer)?;
    archive.read_bytes(&mut physical_buffer)?;

    let virtual_data = deflate_bytes(&virtual_buffer);
    let physical_data = deflate_bytes(&physical_buffer);
    log::info!("Decompressed virtual size: {:?}", virtual_data.len());
    log::info!("Decompressed physical size: {:?}", physical_data.len());

    let mut graphics_archive = FMemoryArchive::new(physical_data);
    // graphics_archive.set_position(0x50000000)?;

    let vft = graphics_archive.read_ulong()?;
    let pages_info_pointer = graphics_archive.read_ulong()?;
    log::info!("VFT: {}", vft);
    log::info!("Pages info pointer: {}", pages_info_pointer);

    Ok(())
}
