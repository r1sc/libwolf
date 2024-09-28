use std::{env::args, fs::File, io::{BufReader, Read}};

use byteorder::{LittleEndian, ReadBytesExt};

fn main() {
    let filename = args().nth(1).unwrap();
    let mut reader = BufReader::new(File::open(&filename).unwrap());

    let mut all_data = Vec::new();

    while let Ok(kind) = reader.read_u8() {
        let len = reader.read_u16::<LittleEndian>().unwrap();
        let mut data = vec![0; len as usize];
        reader.read_exact(&mut data).unwrap();
        if kind == 0xA0 {
            all_data.append(&mut data[3..data.len()-1].to_vec());
        }
    }

    println!("Length: {}", all_data.len());
    let output_filename = format!("{}.bin", filename);
    std::fs::write(output_filename, all_data).unwrap();
}
