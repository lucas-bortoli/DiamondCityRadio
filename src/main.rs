use ringbuffer_sound::RingBufferSound;
use state_resumer::determine_expected_current_state;
use station_data::{POLL_BUFFER_SIZE_BYTES, SAMPLE_RATE, SoundFile, Station};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    ops::DerefMut,
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};

mod ringbuffer_sound;
mod state_resumer;
mod station_data;

fn calculate_sleep_duration(buffer_size_samples: u32) -> Duration {
    let time_per_buffer_secs = buffer_size_samples as f32 / SAMPLE_RATE as f32;
    Duration::from_secs_f32(time_per_buffer_secs)
}

fn main() {
    let (tx, rx) = mpsc::channel();

    let station = Station::from_file("./diamond_city_radio/radio.yaml")
        .expect("Failed to parse station file");
    let station_thread_clone = station.clone();
    thread::spawn(move || {
        let station = station_thread_clone;

        fn play_wav_blocking<S: SoundFile>(
            tx: &Sender<Box<[u8; POLL_BUFFER_SIZE_BYTES]>>,
            sound: &S,
            start_time_ms: u64,
        ) {
            let mut file =
                File::open(sound.source_filename()).expect("Can't open file for playback");

            file.seek(SeekFrom::Start(sound.time_to_byte_offset(start_time_ms)))
                .expect("Can't seek file");

            loop {
                const BUFFER_SAMPLE_COUNT: u32 = SAMPLE_RATE / 10; // 100ms de áudio
                let mut audio_buffer = Box::new([0 as u8; POLL_BUFFER_SIZE_BYTES]);

                let bytes_read = file
                    .read(audio_buffer.deref_mut())
                    .expect("Can't read sound file");

                if bytes_read == 0 {
                    println!("EOF.");
                    break;
                }

                if tx.send(audio_buffer).is_err() {
                    println!("Can't send data. Consumer stopped?");
                    break;
                }

                thread::sleep(calculate_sleep_duration(BUFFER_SAMPLE_COUNT));
            }
        }

        loop {
            let (current_state, current_state_elapsed, current_state_total) =
                determine_expected_current_state(&station);

            match current_state {
                state_resumer::StationExpectedState::SilenceInterval => {
                    thread::sleep(Duration::from_millis(
                        current_state_total - current_state_elapsed,
                    ));
                }
                state_resumer::StationExpectedState::TrackNarrationBefore {
                    narration,
                    imminent_track: _,
                } => {
                    if let Some(narration_val) = narration {
                        println!("Travis [before]: {}", narration_val.content);
                        play_wav_blocking(&tx, &narration_val, current_state_elapsed);
                    }
                }
                state_resumer::StationExpectedState::Track { track } => {
                    println!("Now playing: {} [{} ms]", track.title, track.duration_ms());
                    play_wav_blocking(&tx, &track, current_state_elapsed);
                }
                state_resumer::StationExpectedState::TrackNarrationAfter {
                    previous_track: _,
                    narration,
                } => {
                    if let Some(narration_val) = narration {
                        println!("Travis [after]: {}", narration_val.content);
                        play_wav_blocking(&tx, &narration_val, current_state_elapsed);
                    }
                }
            }
        }
    });

    let (mut manager, _backend) = awedio::start().expect("Can't start awedio");
    let sound = Box::new(RingBufferSound::new());
    let handle = sound.spawn_thread_and_join(rx);
    manager.play(sound);
    handle.join().unwrap();
}
