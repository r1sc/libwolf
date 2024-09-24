mod audiot;

use std::{env::args, io::Cursor};

use audiot::read_audiohed;
use byteorder::{LittleEndian, ReadBytesExt};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

const STARTMUSIC: usize = 261;

fn main() {
    let music_number = args()
        .nth(1)
        .expect("usage: wolf_audio <music number>")
        .parse::<usize>()
        .expect("<music number> must be a number");

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");

    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");
    
    let supported_config = supported_configs_range
        .next()
        .expect("no supported config?!")
        .with_max_sample_rate();

    let config = supported_config.config();
    println!("Using output config: {:?}", supported_config);

    let song_freq_hz = 700;
    let sample_rate = supported_config.sample_rate().0;
    let opl_ticks_per_sample = sample_rate / song_freq_hz;

    let mut a = audiot::AudioT::new();

    let audio_head =
        read_audiohed(&mut std::fs::File::open(r"C:\classic\WOLF3D\AUDIOHED.WL6").unwrap())
            .unwrap();

    let audio_data = audiot::read_audiot_chunk(
        &mut std::fs::File::open(r"C:\classic\WOLF3D\AUDIOT.WL6").unwrap(),
        STARTMUSIC + music_number,
        &audio_head,
    )
    .unwrap();

    let mut audio_cursor = Cursor::new(audio_data);
    let audio_len = audio_cursor.read_u16::<LittleEndian>().unwrap();

    let mut num_samples_ready = 0;
    let mut time_counter: u32 = 0;
    let mut next_command_at = 0;

    let stream = match device.build_output_stream(
        &config,
        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
            let mut buffer_pos = 0;

            while buffer_pos < data.len() {
                loop {
                    if next_command_at > time_counter {
                        break;
                    }

                    if audio_cursor.position() >= audio_len as u64 {
                        audio_cursor.set_position(2);
                        next_command_at = 0;
                        time_counter = 0;
                        break;
                    }

                    let reg = audio_cursor.read_u8().unwrap();
                    let value = audio_cursor.read_u8().unwrap();
                    let delay = audio_cursor.read_u16::<LittleEndian>().unwrap();

                    next_command_at = time_counter + (delay as u32);

                    a.send_data(reg as u32, value);
                }

                time_counter += 1;
                num_samples_ready += opl_ticks_per_sample as usize;

                while num_samples_ready > 0 {
                    let sample = a.get_sample();

                    for _ in 0..config.channels as usize {
                        data[buffer_pos] = sample;
                        buffer_pos += 1;
                        if buffer_pos >= data.len() {
                            break;
                        }
                    }
                    
                    num_samples_ready -= 1;

                    if buffer_pos >= data.len() {
                        break;
                    }
                }
            }
        },
        move |_err| {
            // react to errors here.
        },
        None, // None=blocking, Some(Duration)=timeout
    ) {
        Ok(stream) => stream,
        Err(err) => panic!("An error occurred: {}", err),
    };

    stream.play().unwrap();

    // Wait for enter
    println!("Press ENTER to exit");
    std::io::stdin().read_line(&mut String::new()).unwrap();
}
