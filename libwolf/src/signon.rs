const SIGNON: &[u8] = include_bytes!("../SIGNON.BIN");

pub fn draw(output_buffer: &mut [u32], palette: &[u32]) {
    let width = 320 / 4;
    for plane in 0..4 {
        for y in 0..200 {
            for x in 0..width {
                let offset = y * 320 + x + plane * width;
                output_buffer[offset] = palette[SIGNON[offset] as usize];
            }
        }
    }
}