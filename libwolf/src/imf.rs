use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};
// use emu8950_sys::{OPL_calc, OPL_new, OPL_writeReg, OPL};

use crate::audiot::{read_audiohed, read_audiot_chunk};

const STARTMUSIC: usize = 261;
const SONG_FREQ_HZ: u32 = 700;

pub struct Imf {
    opl: opl3_rs::Opl3Device,
    num_samples_ready: usize,
    time_counter: u32,
    next_command_at: u32,
    audio_len: usize,
    audio_cursor: Cursor<Vec<u8>>,
    opl_ticks_per_sample: u32,
}

impl Imf {
    pub fn new(
        wolf3d_path: &str,
        music_number: usize,
        output_sample_rate: u32,
    ) -> std::io::Result<Self> {
        // let opl = unsafe { OPL_new(3579545, output_sample_rate) };
        let opl = opl3_rs::Opl3Device::new(output_sample_rate);

        let audio_head = read_audiohed(&mut std::fs::File::open(format!(
            "{}/AUDIOHED.WL6",
            wolf3d_path
        ))?)?;

        let audio_data = read_audiot_chunk(
            &mut std::fs::File::open(format!("{}/AUDIOT.WL6", wolf3d_path))?,
            STARTMUSIC + music_number,
            &audio_head,
        )?;

        let mut audio_cursor = Cursor::new(audio_data);
        let audio_len = audio_cursor.read_u16::<LittleEndian>()? as usize;

        Ok(Self {
            opl,
            num_samples_ready: 0,
            next_command_at: 0,
            time_counter: 0,
            audio_len,
            audio_cursor,
            opl_ticks_per_sample: output_sample_rate / SONG_FREQ_HZ,
        })
    }

    pub fn fill_audio_buffer(
        &mut self,
        data: &mut [i16],
        num_channels: u32,
    ) -> std::io::Result<()> {
        let mut buffer_pos = 0;

        while buffer_pos < data.len() {
            loop {
                if self.next_command_at > self.time_counter {
                    break;
                }

                if self.audio_cursor.position() >= self.audio_len as u64 {
                    self.audio_cursor.set_position(2);
                    self.next_command_at = 0;
                    self.time_counter = 0;
                    break;
                }

                let reg = self.audio_cursor.read_u8().unwrap();
                let value = self.audio_cursor.read_u8().unwrap();
                let delay = self.audio_cursor.read_u16::<LittleEndian>().unwrap();

                self.next_command_at = self.time_counter + delay as u32;

                // unsafe { OPL_writeReg(self.opl, reg as u32, value) };
                self.opl
                    .write_register(reg, value, opl3_rs::OplRegisterFile::Primary, false);
            }

            self.time_counter += 1;
            self.num_samples_ready += self.opl_ticks_per_sample as usize;

            while self.num_samples_ready > 0 {
                // let sample = unsafe { OPL_calc(self.opl) } * 6; // Increase gain otherwise music is too quiet
                let mut samples: [i16; 2] = [0, 0];
                self.opl.generate(&mut samples).unwrap();

                for i in 0..num_channels  {
                    data[buffer_pos] = samples[(i % num_channels) as usize];
                    buffer_pos += 1;
                    if buffer_pos >= data.len() {
                        break;
                    }
                }

                self.num_samples_ready -= 1;

                if buffer_pos >= data.len() {
                    break;
                }
            }
        }

        Ok(())
    }
}
