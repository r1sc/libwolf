use std::io::{Cursor, Read, Seek};

use byteorder::{LittleEndian, ReadBytesExt};

pub struct PCMInfo {
    pub chunk_start_index: u16,
    pub length: u16,
}

pub struct VSWAPArchive {
    pub wall_chunks: Vec<Vec<u8>>,
    pub sprite_chunks: Vec<Vec<u8>>,
    pub raw_pcm_chunks: Vec<Vec<u8>>,
}

impl VSWAPArchive {
    pub fn open<R: Read + Seek>(reader: &mut R) -> std::io::Result<Self> {
        let chunks_in_file = reader.read_i16::<LittleEndian>()? as usize;
        let pm_sprite_start = reader.read_i16::<LittleEndian>()? as usize;
        let pm_sound_start = reader.read_i16::<LittleEndian>()? as usize;

        let chunk_offsets = (0..chunks_in_file)
            .map(|_| reader.read_u32::<LittleEndian>())
            .collect::<std::io::Result<Vec<_>>>()?;

        let chunk_lengths = (0..chunks_in_file)
            .map(|_| reader.read_u16::<LittleEndian>())
            .collect::<std::io::Result<Vec<_>>>()?;

        let mut wall_chunks = Vec::new();
        let mut sprite_chunks = Vec::new();
        let mut sound_chunks = Vec::new();

        // Skip last chunk, that's special
        for i in 0..chunks_in_file - 1 {
            reader
                .seek(std::io::SeekFrom::Start(chunk_offsets[i] as u64))
                .unwrap();

            let mut buffer = vec![0; chunk_lengths[i] as usize];
            reader.read_exact(&mut buffer).unwrap();

            if i < pm_sprite_start {
                wall_chunks.push(buffer);
            } else if i < pm_sound_start {
                sprite_chunks.push(buffer);
            } else {
                sound_chunks.push(buffer);
            }
        }

        reader
            .seek(std::io::SeekFrom::Start(
                chunk_offsets[chunks_in_file - 1] as u64,
            ))
            .unwrap();

        // Read the pcm info, which is the last "sound" chunk in the file
        let mut pcm_infos = Vec::new();
        let last_sound_chunk_len = chunk_lengths[chunks_in_file - 1] / 4;

        for _ in 0..last_sound_chunk_len {
            pcm_infos.push(PCMInfo {
                chunk_start_index: reader.read_u16::<LittleEndian>().unwrap(),
                length: reader.read_u16::<LittleEndian>().unwrap(),
            });
        }

        // Then build the actual pcm chunks, some spanning multiple chunks
        let mut raw_pcm_chunks = Vec::new();

        for i in 0..pcm_infos.len()-1 {
            let pcm_info = &pcm_infos[i];
            let next_chunk_info = &pcm_infos[i + 1];

            let chunks = &sound_chunks
                [pcm_info.chunk_start_index as usize..next_chunk_info.chunk_start_index as usize];

            // From the specified chunk in the pcm_info to the next chunk,
            // join the chunks together and take the first pcm_info.length bytes

            // Why not read the entire sound data as one big buffer?
            // Because the raw chunk data is padded, and we should not play the padding

            let raw_pcm_chunk = chunks
                .iter()
                .flatten()
                .take(pcm_info.length as usize)
                .copied()
                .collect::<Vec<_>>();

            raw_pcm_chunks.push(raw_pcm_chunk);
        }

        if cfg!(debug_assertions) {
            println!(
                "Num walls: {}, num sprites: {}, num sounds: {}",
                wall_chunks.len(),
                sprite_chunks.len(),
                sound_chunks.len()
            );
        }

        Ok(Self {
            wall_chunks,
            sprite_chunks,
            raw_pcm_chunks,
        })
    }

    pub fn rasterize_wall(&self, wall_num: usize, palette: &[u32], output_buffer: &mut [u32]) {
        let wall_data: &[u8] = &self.wall_chunks[wall_num];

        for x in 0..64 {
            for y in 0..64 {
                let pix = wall_data[x * 64 + y] as usize;
                output_buffer[y * 64 + x] = palette[pix];
            }
        }
    }

    pub fn rasterize_sprite(&self, sprite_num: usize, palette: &[u32], output_buffer: &mut [u32]) {
        let sprite_data: &[u8] = &self.sprite_chunks[sprite_num];
        let mut sprite_reader = Cursor::new(&sprite_data);

        let left_pix = sprite_reader.read_u16::<LittleEndian>().unwrap();
        let right_pix = sprite_reader.read_u16::<LittleEndian>().unwrap();
        let num_columns = right_pix - left_pix + 1;
        let column_offsets = (0..num_columns)
            .map(|_| sprite_reader.read_u16::<LittleEndian>().unwrap())
            .collect::<Vec<_>>();

        // left pix, right pix, column offsets
        let mut pixel_offset = sprite_reader.position();

        for x in 0..num_columns {
            sprite_reader.set_position(column_offsets[x as usize] as u64);

            loop {
                let mut ending_row = sprite_reader.read_u16::<LittleEndian>().unwrap();
                if ending_row == 0 {
                    // 0 signals the end of a column
                    break;
                }

                let _ = sprite_reader.read_u16::<LittleEndian>().unwrap(); // Skip two bytes, don't know what they are
                let mut starting_row = sprite_reader.read_u16::<LittleEndian>().unwrap();

                // I don't know why these are double the size of the actual rows
                ending_row >>= 1;
                starting_row >>= 1;

                for y in starting_row..ending_row {
                    let pix = sprite_data[pixel_offset as usize] as usize;
                    output_buffer[y as usize * 64 + x as usize] = palette[pix];

                    pixel_offset += 1;
                }
            }
        }
    }
}
