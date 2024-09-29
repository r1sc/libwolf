use libwolf::{gr, vswap::VSWAPArchive, wl6_igrab};
use minifb::{Key, Window, WindowOptions};
use std::{env::args, fs::File, io::BufReader};

// fn blit(
//     src_data: &[u8],
//     src_width: u16,
//     src_height: u16,
//     dest_x: usize,
//     dest_y: usize,
//     dest_data: &mut [u32],
//     palette: &[u32],
// ) {
//     for j in 0..src_height as usize {
//         for i in 0..src_width as usize {
//             let color_index = src_data[j * src_width as usize + i] as usize;
//             dest_data[(dest_y + j) * 320 + dest_x + i] = palette[color_index];
//         }
//     }
// }

struct ScratchBuffer<'a> {
    data: Vec<u8>,
    palette: &'a [u32],
}

impl<'a> ScratchBuffer<'a> {
    pub fn new(palette: &'a [u32]) -> Self {
        ScratchBuffer {
            data: vec![0; 320 * 200],
            palette,
        }
    }

    pub fn blit(
        &self,
        src_width: u16,
        src_height: u16,
        dest_data: &mut [u32],
        dest_x: usize,
        dest_y: usize,
        rotate: bool,
    ) {
        for y in 0..src_height as usize {
            for x in 0..src_width as usize {
                let (x, y) = if rotate { (y, x) } else { (x, y) };
                let color_index = self.data[y * 320 + x] as usize;
                dest_data[(dest_y + y) * 320 + dest_x + x] = self.palette[color_index];
            }
        }
    }
}

fn main() {
    let asset_number = args()
        .nth(1)
        .expect("usage: wolf_audio <music number>")
        .parse::<usize>()
        .expect("<music number> must be a number");

    let mut palette_u32 = vec![0; 256];
    let brightness = 2;

    for i in 0..256 {
        let r = libwolf::GAMEPAL[i * 3] as u32;
        let g = libwolf::GAMEPAL[i * 3 + 1] as u32;
        let b = libwolf::GAMEPAL[i * 3 + 2] as u32;

        palette_u32[i] = (r << brightness << 16) | (g << brightness << 8) | b << brightness;
    }

    let mut screen_buffer_u32: Vec<u32> = vec![0; 320 * 200];

    let mut scratch_buffer = ScratchBuffer::new(&palette_u32);

    libwolf::signon::draw(&mut scratch_buffer.data);
    scratch_buffer.blit(320, 200, &mut screen_buffer_u32, 0, 0, false);

    let wolf_base_path = r"c:\classic\wolf3d";

    let mut gr = gr::GrArchive::new(wolf_base_path);

    let pic = gr.load_pic(wl6_igrab::GraphicNum::L_BJWINSPIC).unwrap();
    pic.draw(&mut scratch_buffer.data);
    scratch_buffer.blit(pic.size.width, pic.size.height, &mut screen_buffer_u32, 200, 50, false);

    let mut reader = BufReader::new(File::open(format!("{}/vswap.wl6", wolf_base_path)).unwrap());
    let vswap = VSWAPArchive::open(&mut reader).unwrap();

    let mut current_sprite = 0;

    vswap.rasterize_wall(18, &mut scratch_buffer.data);
    scratch_buffer.blit(64, 64, &mut screen_buffer_u32, 0, 0, true);

    vswap.rasterize_sprite(54, &mut scratch_buffer.data);
    scratch_buffer.blit(64, 64, &mut screen_buffer_u32, 0, 0, false);

    let output_sample_rate = 44100;
    let num_streaming_buffers = 4;
    let music_buffer_size = 12000;
    let num_channels = 2; // Stereo

    let mut imf = libwolf::imf::Imf::new(wolf_base_path, asset_number, output_sample_rate).unwrap();

    let mut mixer = libwolf::mixer::Mixer::new(num_streaming_buffers);
    let mut music_buffer: Vec<i16> = vec![0; music_buffer_size * num_channels as usize];

    for _ in 0..num_streaming_buffers {
        imf.fill_audio_buffer(&mut music_buffer, num_channels)
            .unwrap();
        mixer.queue_music_data(output_sample_rate, num_channels, &music_buffer);
    }

    let pcm_sound = mixer.load_raw_pcm(7000, &vswap.raw_pcm_chunks[asset_number]);
    mixer.play_pcm_buffer(&pcm_sound, 0.2, true);

    let scale = 2;
    let mut window = Window::new(
        "Test - ESC to exit",
        320 * scale,
        240 * scale,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    window.set_target_fps(60);

    while window.is_open() {
        if window.is_key_pressed(Key::Right, minifb::KeyRepeat::Yes)
            && current_sprite < vswap.sprite_chunks.len() - 1
        {
            current_sprite += 1;
            // screen_buffer_u8.fill(0);
            // vswap.rasterize_sprite(current_sprite, &mut screen_buffer_u8);
        } else if window.is_key_pressed(Key::Left, minifb::KeyRepeat::Yes) && current_sprite > 0 {
            current_sprite -= 1;
            // screen_buffer_u8.fill(0);
            // vswap.rasterize_sprite(current_sprite, &mut screen_buffer_u8);
        }

        window
            .update_with_buffer(&screen_buffer_u32, 320, 200)
            .unwrap();

        // Process music
        mixer.unqueue_processed_buffers();

        if mixer.get_num_empty_music_buffers() > 0 {
            // mixer.print_buffer_queue();

            imf.fill_audio_buffer(&mut music_buffer, num_channels)
                .unwrap();
            mixer.queue_music_data(output_sample_rate, num_channels, &music_buffer);
        }
    }
}
