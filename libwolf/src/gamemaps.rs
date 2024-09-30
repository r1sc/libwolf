use std::{
    fs::File,
    io::{BufReader, Read},
};

use byteorder::{LittleEndian, ReadBytesExt};

pub struct Gamemaps {
    pub plane0: Vec<u8>,
    pub plane1: Vec<u8>,
    pub plane2: Vec<u8>,
    pub width: u16,
    pub height: u16,
    pub name: String,
}

impl Gamemaps {
    pub fn new(path: &str) -> Vec<Self> {
        let mut maphead = BufReader::new(File::open(format!("{}/MAPHEAD.WL6", path)).unwrap());
        let mut gamemaps = vec![];
        let _ = File::open(format!("{}/GAMEMAPS.WL6", path))
            .unwrap()
            .read_to_end(&mut gamemaps);

        let magic = maphead.read_u16::<LittleEndian>().unwrap();

        let mut result = vec![];

        for _ in 0..100 {
            let ptr = maphead.read_i32::<LittleEndian>().unwrap();
            if ptr == 0 {
                continue;
            }

            let mut data = BufReader::new(gamemaps.get(ptr as usize..ptr as usize + 38).unwrap());
            let off_plane0 = data.read_i32::<LittleEndian>().unwrap();
            let off_plane1 = data.read_i32::<LittleEndian>().unwrap();
            let off_plane2 = data.read_i32::<LittleEndian>().unwrap();
            let len_plane0 = data.read_u16::<LittleEndian>().unwrap();
            let len_plane1 = data.read_u16::<LittleEndian>().unwrap();
            let len_plane2 = data.read_u16::<LittleEndian>().unwrap();
            let width = data.read_u16::<LittleEndian>().unwrap();
            let height = data.read_u16::<LittleEndian>().unwrap();
            let mut name = String::new();
            let _ = data.read_to_string(&mut name);

            let plane0 = Self::get_plane_data(&gamemaps, off_plane0, len_plane0, magic);
            let plane1 = Self::get_plane_data(&gamemaps, off_plane1, len_plane1, magic);
            let plane2 = Self::get_plane_data(&gamemaps, off_plane2, len_plane2, magic);

            result.push(Self {
                plane0,
                plane1,
                plane2,
                width,
                height,
                name,
            });
        }

        result
    }

    fn get_plane_data(data: &[u8], offset: i32, len: u16, magic: u16) -> Vec<u8> {
        let plane_data = data
            .get(offset as usize..offset as usize + len as usize)
            .unwrap();
        let plane_data = carmack_expand(plane_data);
        let plane_data = rlew_expand(&plane_data, magic);

        plane_data
    }
}

fn carmack_expand(compressed: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut buf = BufReader::new(compressed);

    let decompressed_size = buf.read_u16::<LittleEndian>().unwrap() as usize;
    let mut length = decompressed_size / 2;

    while length > 0 {
        let ch = buf.read_u16::<LittleEndian>().unwrap();
        let chhigh = ch >> 8;
        if chhigh == 0xA7 {
            let count = ch & 0xFF;
            if count == 0 {
                let ch = buf.read_u8().unwrap();
                result.push(ch);
                length -= 1;
            } else {
                let offset = buf.read_u8().unwrap() as usize;
                let mut copyptr = result.len() - (offset * 2);
                length -= count as usize;
                for _ in 0..count * 2 {
                    result.push(result[copyptr]);
                    copyptr += 1;
                }
            }
        } else if chhigh == 0xA8 {
            let count = ch & 0xFF;
            if count == 0 {
                let ch = buf.read_u8().unwrap();
                result.push(ch);
                length -= 1;
            } else {
                let offset = buf.read_u16::<LittleEndian>().unwrap() as usize;
                let mut copyptr = offset * 2;
                length -= count as usize;
                for _ in 0..count * 2 {
                    result.push(result[copyptr]);
                    copyptr += 1;
                }
            }
        } else {
            result.push((ch & 0xFF) as u8);
            result.push((ch >> 8) as u8);
            length -= 1;
        }
    }

    assert_eq!(result.len(), decompressed_size);

    result
}

fn rlew_expand(compressed: &[u8], rlewtag: u16) -> Vec<u8> {
    let mut result = Vec::new();
    let mut buf = BufReader::new(compressed);

    let decompressed_size = buf.read_u16::<LittleEndian>().unwrap() as usize;

    while result.len() < decompressed_size {
        let value = buf.read_u16::<LittleEndian>().unwrap();
        if value != rlewtag {
            result.push((value & 0xFF) as u8);
            result.push((value >> 8) as u8);
        } else {
            let count = buf.read_u16::<LittleEndian>().unwrap();
            let value = buf.read_u16::<LittleEndian>().unwrap();
            for _ in 0..count {
                result.push((value & 0xFF) as u8);
                result.push((value >> 8) as u8);
            }
        }
    }

    assert_eq!(result.len(), decompressed_size);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress_carmack_compression_near_pointers() {
        assert_eq!(
            carmack_expand(&[
                0x10, 0x00, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x02, 0xa7, 0x02, 0x01, 0x02, 0x03,
                0x04, 0x05, 0x06,
            ]),
            vec![
                0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0xcc, 0xdd, 0xee, 0xff, 0x01, 0x02, 0x03, 0x04,
                0x05, 0x06
            ]
        );
    }

    #[test]
    fn test_dcmcnp() {
        assert_eq!(
            carmack_expand(&[
                22, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
                0x04, 0xA7, 0x06, 0x00, 0x01
            ]),
            [
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x00, 0x01,
                0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x00, 0x01
            ]
        );
    }

    #[test]
    fn test_decompress_carmack_compression_far_pointers() {
        assert_eq!(
            carmack_expand(&[
                0x10, 0x00, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x02, 0xa8, 0x01, 0x00, 0x01, 0x02,
                0x03, 0x04, 0x05, 0x06,
            ]),
            vec![
                0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0xcc, 0xdd, 0xee, 0xff, 0x01, 0x02, 0x03, 0x04,
                0x05, 0x06
            ]
        );
    }

    #[test]
    fn test_decompress_rlew() {
        assert_eq!(
            rlew_expand(&[0x04, 0x00, 0xFE, 0xFE, 0x02, 0x00, 0x03, 0x04], 0xFEFE),
            vec![0x03, 0x04, 0x03, 0x04]
        );
    }

    #[test]
    fn test_decompress_rlew_flag_word() {
        assert_eq!(
            rlew_expand(&[0x02, 0x00, 0xFE, 0xFE, 0x01, 0x00, 0xFE, 0xFE], 0xFEFE),
            vec![0xFE, 0xFE]
        );
    }
}
