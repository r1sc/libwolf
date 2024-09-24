use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek};

pub fn read_audiohed<R: Read + Seek>(reader: &mut R) -> std::io::Result<Vec<u32>> {
    let mut offsets = Vec::new();

    while let Ok(offset) = reader.read_u32::<LittleEndian>() {
        offsets.push(offset);
    }

    Ok(offsets)
}

pub fn read_audiot_chunk<R: Read + Seek>(
    reader: &mut R,
    offset_index: usize,
    offsets: &[u32],
) -> std::io::Result<Vec<u8>> {
    let offset = offsets[offset_index] as u64;
    let next_offset = offsets[offset_index + 1] as u64;
    let len = next_offset - offset;

    reader.seek(std::io::SeekFrom::Start(offset))?;

    let mut buffer = vec![0; len as usize];
    reader.read_exact(&mut buffer)?;

    Ok(buffer)
}
