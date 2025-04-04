use process_priority::set_high_priority;
use rocket::{
    http::ContentType,
    response::{
        content::{RawCss, RawHtml},
        stream::ByteStream,
    },
    tokio::sync::broadcast,
};
use state_resumer::determine_expected_current_state;
use station_data::{
    BIT_DEPTH, CHANNEL_COUNT, POLL_BUFFER_SIZE_BYTES, SAMPLE_RATE, SoundFile, Station,
};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    ops::DerefMut,
    sync::Arc,
    thread,
    time::Duration,
};

#[macro_use]
extern crate rocket;

mod process_priority;
mod state_resumer;
mod station_data;

#[get("/")]
fn index() -> RawHtml<&'static [u8]> {
    return RawHtml(include_bytes!("ui.html"));
}

#[get("/ui.css")]
fn stylesheet() -> RawCss<&'static [u8]> {
    return RawCss(include_bytes!("ui.css"));
}

struct AudioBroadcaster {
    sender: broadcast::Sender<Box<[u8; POLL_BUFFER_SIZE_BYTES]>>,
}

pub fn wav_header(sample_rate: u32, bits_per_sample: u16, num_channels: u16) -> [u8; 44] {
    let block_align = num_channels * (bits_per_sample / 8) as u16;
    let byte_rate = sample_rate * block_align as u32;

    let mut header = [0u8; 44];

    // RIFF Chunk
    header[..4].copy_from_slice(b"RIFF"); // ChunkID
    header[4..8].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // ChunkSize (unknown size)
    header[8..12].copy_from_slice(b"WAVE"); // Format

    // fmt Chunk
    header[12..16].copy_from_slice(b"fmt "); // Subchunk1ID
    header[16..20].copy_from_slice(&16u32.to_le_bytes()); // Subchunk1Size (PCM)
    header[20..22].copy_from_slice(&1u16.to_le_bytes()); // AudioFormat (1 = PCM)
    header[22..24].copy_from_slice(&num_channels.to_le_bytes()); // NumChannels
    header[24..28].copy_from_slice(&sample_rate.to_le_bytes()); // SampleRate
    header[28..32].copy_from_slice(&byte_rate.to_le_bytes()); // ByteRate
    header[32..34].copy_from_slice(&block_align.to_le_bytes()); // BlockAlign
    header[34..36].copy_from_slice(&bits_per_sample.to_le_bytes()); // BitsPerSample

    // data Chunk
    header[36..40].copy_from_slice(b"data"); // Subchunk2ID
    header[40..44].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // Subchunk2Size (streaming mode)

    header
}

#[get("/diamondcity")]
fn station_diamondcity(
    state: &rocket::State<Arc<AudioBroadcaster>>,
) -> (ContentType, ByteStream![Vec<u8>]) {
    let mut rx = state.sender.subscribe();

    (
        ContentType::new("audio", "wav"),
        ByteStream! {
            yield Vec::from_iter(wav_header(SAMPLE_RATE, BIT_DEPTH as u16, CHANNEL_COUNT as u16).into_iter());

            while let Ok(chunk) = rx.recv().await {

                yield Vec::from_iter(chunk.into_iter());
                //yield chunk.deref();
            }
        },
    )
}

fn calculate_sleep_duration(buffer_size_samples: u32) -> Duration {
    let time_per_buffer_secs = buffer_size_samples as f32 / SAMPLE_RATE as f32;
    Duration::from_secs_f32(time_per_buffer_secs)
}

#[launch]
fn rocket() -> _ {
    set_high_priority();

    let (tx, _) = broadcast::channel::<Box<[u8; POLL_BUFFER_SIZE_BYTES]>>(8);

    let broadcaster = Arc::new(AudioBroadcaster { sender: tx.clone() });

    let station = Station::from_file("./diamond_city_radio/radio.yaml")
        .expect("Failed to parse station file");
    let station_thread_clone = station.clone();

    // audio source thread
    thread::spawn(move || {
        let station = station_thread_clone;

        fn play_wav_blocking<S: SoundFile>(
            tx: &broadcast::Sender<Box<[u8; POLL_BUFFER_SIZE_BYTES]>>,
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
                    //println!("Can't send data. No audio consumers?");
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

    rocket::build()
        .manage(broadcaster)
        .mount("/", routes![index, stylesheet])
        .mount("/station", routes![station_diamondcity])
}
