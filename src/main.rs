use ringbuffer_sound::RingBufferSound;
use station_data::{BIT_DEPTH, CHANNEL_COUNT, POLL_BUFFER_SIZE_BYTES, SAMPLE_RATE, Station, Track};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    ops::DerefMut,
    sync::mpsc,
    thread,
    time::Duration,
};

mod ringbuffer_sound;
mod state_resumer;
mod station_data;

fn calculate_sleep_duration(buffer_size_samples: usize) -> Duration {
    let time_per_buffer_secs = buffer_size_samples as f32 / SAMPLE_RATE as f32;
    Duration::from_secs_f32(time_per_buffer_secs)
}

fn announce_track(track: &Track) {
    let duration = track.duration_s();
    println!(
        "{} | {:02}:{:02}",
        track.title,
        (duration / 60.0).floor() as u32,
        (duration % 60.0).floor() as u32
    );
}

fn main() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let station = Station::from_file("./diamond_city_radio/radio.yaml")
            .expect("Failed to parse station file");

        println!("{:#?}", station);

        let resume_info = station.determine_current_track_for_resuming();
        let resume_offset = resume_info.track.time_to_byte_offset(resume_info.seek_ms);

        let mut track = resume_info.track;
        let mut track_file = File::open("./diamond_city_radio/".to_string() + &track.source)
            .expect("Can't open track source");

        println!("Resuming! {}ms", resume_info.seek_ms);
        announce_track(track);

        track_file
            .seek(SeekFrom::Start(44 + resume_offset))
            .expect("Can't seek track source for resuming");

        loop {
            const BUFFER_SAMPLE_COUNT: u32 = SAMPLE_RATE / 10; // 100ms de áudio
            let mut audio_buffer = Box::new([0 as u8; POLL_BUFFER_SIZE_BYTES]);

            let bytes_read = track_file
                .read(audio_buffer.deref_mut())
                .expect("Can't read track source");

            // println!("{}: Read {} bytes", track.source, bytes_read);

            if tx.send(audio_buffer).is_err() {
                println!("Can't send data. Consumer stopped?");
                break;
            }

            if bytes_read == 0 {
                println!("EOF.");
                thread::sleep(Duration::from_millis(200));
                // final do arquivo
                let next_track = station.determine_current_track_for_resuming();
                track = next_track.track;
                announce_track(track);
                track_file =
                    File::open("./diamond_city_radio/".to_string() + &next_track.track.source)
                        .expect("Can't open next track source");
                track_file.seek(SeekFrom::Start(44)).unwrap();
            } else {
                thread::sleep(calculate_sleep_duration(BUFFER_SAMPLE_COUNT as usize));
            }
        }
    });

    let (mut manager, _backend) = awedio::start().expect("Can't start awedio");
    let sound = Box::new(RingBufferSound::new());
    let handle = sound.spawn_thread_and_join(rx);
    manager.play(sound);
    handle.join().unwrap();
}
