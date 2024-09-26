use std::{collections::VecDeque, ptr::null};

use openal_sys::*;

pub struct PCMBuffer(ALuint);

pub struct Mixer {
    device: *mut ALCdevice,
    context: *mut ALCcontext,
    static_source: ALuint,
    music_source: ALuint,
    streaming_queue: VecDeque<ALuint>,
}

impl Drop for Mixer {
    fn drop(&mut self) {
        unsafe {
            alcDestroyContext(self.context);
            alcCloseDevice(self.device);
        }
    }
}

impl Mixer {
    pub fn new(num_music_buffers: usize) -> Self {
        let device = unsafe { alcOpenDevice(null()) };
        let context = unsafe { alcCreateContext(device, null()) };

        unsafe {
            alcMakeContextCurrent(context);
        }

        let (static_source, music_source) = unsafe {
            let mut sources: [ALuint; 2] = [0, 0];
            alGenSources(2, sources.as_mut_ptr());
            (sources[0], sources[1])
        };

        let streaming_queue = VecDeque::from(unsafe {
            let mut buffers = vec![0; num_music_buffers];
            alGenBuffers(num_music_buffers as ALsizei, buffers.as_mut_ptr());
            buffers
        });

        Self {
            device,
            context,
            static_source,
            music_source,
            streaming_queue,
        }
    }

    pub fn get_num_empty_music_buffers(&self) -> usize {
        self.streaming_queue.len()
    }

    pub fn unqueue_processed_buffers(&mut self) {
        unsafe {
            let mut num_processed: ALint = 0;
            alGetSourcei(self.music_source, AL_BUFFERS_PROCESSED, &mut num_processed);

            if num_processed == 0 {
                return;
            }

            let mut unqueued_buffers: [ALuint; 16] = [0; 16];
            alSourceUnqueueBuffers(
                self.music_source,
                num_processed,
                unqueued_buffers.as_mut_ptr(),
            );

            (0..num_processed as usize).for_each(|i| {
                self.streaming_queue.push_back(unqueued_buffers[i]);
            });
        }
    }

    /// Before calling this, make sure get_num_empty_music_buffers() returns at least one
    /// free buffer
    pub fn queue_music_data(&mut self, sample_rate: u32, num_channels: u32, data: &[i16]) {
        let buffer_name = self.streaming_queue.pop_front().unwrap();

        let format = match num_channels {
            1 => AL_FORMAT_MONO16,
            2 => AL_FORMAT_STEREO16,
            _ => panic!("Unsupported number of channels"),
            
        };

        unsafe {
            alBufferData(
                buffer_name,
                format,
                data.as_ptr() as *const std::ffi::c_void,
                (data.len() * 2) as i32,
                sample_rate as i32,
            );

            alSourceQueueBuffers(self.music_source, 1, &buffer_name);

            let mut source_state: ALint = 0;
            alGetSourcei(self.music_source, AL_SOURCE_STATE, &mut source_state);

            if source_state != AL_PLAYING {
                alSourcePlay(self.music_source);
            }
        }
    }

    pub fn load_raw_pcm(&mut self, sample_rate: u32, data: &[u8]) -> PCMBuffer {
        let buffer = unsafe {
            let mut buffer: ALuint = 0;
            alGenBuffers(1, &mut buffer);
            alBufferData(
                buffer,
                AL_FORMAT_MONO8,
                data.as_ptr() as *const std::ffi::c_void,
                data.len() as i32,
                sample_rate as i32,
            );

            buffer
        };

        PCMBuffer(buffer)
    }

    pub fn play_pcm_buffer(&mut self, buffer: &PCMBuffer, volume: f32, looping: bool) {
        unsafe {
            alSourcef(self.static_source, AL_GAIN, volume);

            alSourceStop(self.static_source);
            alSourcei(self.static_source, AL_BUFFER, buffer.0 as i32);
            alSourcei(
                self.static_source,
                AL_LOOPING,
                if looping { AL_TRUE } else { AL_FALSE } as i32,
            );
            alSourcePlay(self.static_source);
        }
    }
}
