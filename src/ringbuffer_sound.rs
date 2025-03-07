use std::{
    ops::Deref,
    sync::{Arc, Mutex, mpsc::Receiver},
    thread::{self, JoinHandle},
};

use crate::station_data::{CHANNEL_COUNT, POLL_BUFFER_SIZE_BYTES, SAMPLE_RATE};
use awedio::{NextSample, Sound};
use bytemuck::cast_slice;

const BUFFER_SIZE: usize = (SAMPLE_RATE * CHANNEL_COUNT) as usize; // quantas samples cabe nesse buffer?

struct RingBuffer {
    contents: Box<[i16; BUFFER_SIZE]>,
    current_position_read: usize,
    current_position_write: usize,
    buffer_full: bool,
}

impl RingBuffer {
    pub fn put(&mut self, sample: i16) {
        if self.buffer_full {
            // buffer is full, move read position forward to make space
            self.current_position_read = (self.current_position_read + 1) % BUFFER_SIZE;
        }
        self.contents[self.current_position_write] = sample;
        self.current_position_write = (self.current_position_write + 1) % BUFFER_SIZE;
        self.buffer_full = self.current_position_write == self.current_position_read;
    }

    pub fn take(&mut self) -> i16 {
        if self.current_position_read == self.current_position_write && !self.buffer_full {
            // buffer is empty
            return 0;
        }
        let sample = self.contents[self.current_position_read];
        self.current_position_read = (self.current_position_read + 1) % BUFFER_SIZE;
        self.buffer_full = false; // once data is read, buffer can't be full
        sample
    }
}
 pub struct RingBufferSound {
    stuff: Arc<Mutex<RingBuffer>>,
}

impl RingBufferSound {
    pub fn new() -> Self {
        RingBufferSound {
            stuff: Arc::new(Mutex::new(RingBuffer {
                contents: Box::new([0i16; BUFFER_SIZE]),
                current_position_read: 0,
                current_position_write: 0,
                buffer_full: false,
            })),
        }
    }

    pub fn spawn_thread_and_join(
        &self,
        rx: Receiver<Box<[u8; POLL_BUFFER_SIZE_BYTES]>>,
    ) -> JoinHandle<()> {
        let stuff = self.stuff.clone();
        thread::spawn(move || {
            loop {
                let audio_frag = rx.recv().expect("Can't receive audio data");
                let samples: &[i16] = cast_slice(audio_frag.deref());
                // println!("Audio data: {} samples", samples.len());

                let mut buffer = stuff.lock().unwrap();

                for sample in samples {
                    buffer.put(*sample);
                }
            }
        })
    }
}

impl Sound for RingBufferSound {
    fn channel_count(&self) -> u16 {
        CHANNEL_COUNT as u16
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    fn next_sample(&mut self) -> Result<NextSample, awedio::Error> {
        let sample = self.stuff.lock().unwrap().take();
        return Ok(NextSample::Sample(sample));
    }

    fn on_start_of_batch(&mut self) {
        // Implement any logic that should run at the start of a batch of samples
    }
}
