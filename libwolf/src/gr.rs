use std::{
    fs::File,
    io::{BufReader, Cursor, Read, Seek, Write},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::wl6_igrab;

pub struct GrArchive {
    huff_dict: [HuffNode; 255],
    gr_starts: Vec<i32>,
    graph_reader: BufReader<File>,
    pic_sizes: Vec<PicSize>,
}

#[derive(Clone, Copy, Default)]
struct HuffNode {
    bit0: u16,
    bit1: u16,
}

#[derive(Clone, Copy)]
pub struct PicSize {
    pub width: u16,
    pub height: u16,
}

impl GrArchive {
    pub fn new(wolf_path: &str) -> Self {
        let mut dict_reader =
            BufReader::new(File::open(format!("{}/VGADICT.WL6", wolf_path)).unwrap());
        let mut huff_dict = [HuffNode::default(); 255];

        for d in huff_dict.iter_mut() {
            d.bit0 = dict_reader.read_u16::<LittleEndian>().unwrap();
            d.bit1 = dict_reader.read_u16::<LittleEndian>().unwrap();
        }

        let mut head_reader =
            BufReader::new(File::open(format!("{}/VGAHEAD.WL6", wolf_path)).unwrap());

        let mut gr_starts = Vec::new();

        for _ in 0..wl6_igrab::NUMCHUNKS + 1 {
            let mut value = head_reader.read_u24::<LittleEndian>().unwrap() as i32;
            if value == 0xFF_FF_FF {
                value = -1;
            }
            gr_starts.push(value);
        }

        let mut this = GrArchive {
            pic_sizes: Vec::new(),
            huff_dict,
            gr_starts,
            graph_reader: BufReader::new(
                File::open(format!("{}/VGAGRAPH.WL6", wolf_path)).unwrap(),
            ),
        };

        let mut pic_sizes_data = Cursor::new(this.expand_chunk(0));
        for _ in 0..wl6_igrab::NUMPICS {
            let width = pic_sizes_data.read_u16::<LittleEndian>().unwrap();
            let height = pic_sizes_data.read_u16::<LittleEndian>().unwrap();
            this.pic_sizes.push(PicSize { width, height });
        }

        this
    }

    pub fn get_pic_size_for_chunk(&self, chunk_index: usize) -> &PicSize {
        &self.pic_sizes[chunk_index - wl6_igrab::STARTPICS]
    }

    fn huff_expand<R: Read, W: Write>(&self, mut compressed_reader: R, dest_writer: &mut W) {
        let head_node = &self.huff_dict[254];
        let mut current_node = head_node;

        let mut current_char = compressed_reader.read_u8().unwrap();
        let mut bit = 1;

        loop {
            let which_bit = if (current_char & bit) == bit {
                current_node.bit1
            } else {
                current_node.bit0
            };

            if which_bit <= 255 {
                dest_writer.write_u8((which_bit & 0xFF) as u8).unwrap();
                current_node = head_node;
            } else {
                current_node = &self.huff_dict[which_bit as usize - 256];
            }

            if bit == 0x80 {
                if let Ok(c) = compressed_reader.read_u8() {
                    current_char = c;
                } else {
                    // Done
                    break;
                }
                bit = 1;
            } else {
                bit <<= 1;
            }
        }
    }

    pub fn expand_chunk(&mut self, chunk_index: usize) -> Vec<u8> {
        let pos = self.gr_starts[chunk_index];
        if pos < 0 {
            panic!("Sparse chunk can't be expanded");
        }

        let mut next = chunk_index + 1;
        while self.gr_starts[next] == -1 {
            next += 1;
        }

        let compressed_size = self.gr_starts[next] - pos;

        self.graph_reader
            .seek(std::io::SeekFrom::Start(pos as u64))
            .unwrap();

        let mut compressed_data = vec![0; compressed_size as usize];
        self.graph_reader.read_exact(&mut compressed_data).unwrap();

        let mut compressed_reader = compressed_data.as_slice();

        let expanded_size =
            if chunk_index >= wl6_igrab::STARTTILE8 && chunk_index < wl6_igrab::STARTEXTERNS {
                //
                // expanded sizes of tile8/16/32 are implicit
                //

                let block = 64;
                let maskblock = 128;

                if chunk_index < wl6_igrab::STARTTILE8M {
                    block * wl6_igrab::NUMTILE8
                } else if chunk_index < wl6_igrab::STARTTILE16 {
                    maskblock * wl6_igrab::NUMTILE8M
                } else if chunk_index < wl6_igrab::STARTTILE32 {
                    maskblock * 4
                } else if chunk_index < wl6_igrab::STARTTILE32M {
                    block * 16
                } else {
                    maskblock * 16
                }
            } else {
                compressed_reader.read_u32::<LittleEndian>().unwrap() as usize
            };

        let mut dest = Vec::with_capacity(expanded_size);
        self.huff_expand(compressed_reader, &mut dest);

        dest
    }
}
