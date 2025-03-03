use core::fmt;
use std::{
    collections::VecDeque,
    time::{SystemTime, UNIX_EPOCH},
};

use frand::Rand;

use crate::station_data::{Narration, STATION_EPOCH, Station, Track};

const SILENCE_INTERVAL_MS: u64 = 200;

pub enum StationExpectedState {
    TrackNarrationBefore {
        narration: Option<Narration>,
        imminent_track: Track,
    },
    Track {
        track: Track,
    },
    TrackNarrationAfter {
        previous_track: Track,
        narration: Option<Narration>,
    },
    SilenceInterval,
}

impl fmt::Display for StationExpectedState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StationExpectedState::SilenceInterval => {
                write!(f, "SilenceInterval[{} ms]", SILENCE_INTERVAL_MS)
            }
            StationExpectedState::Track { track } => {
                write!(
                    f,
                    "Track[{}, {} ms]",
                    track.title.get(0..8).unwrap_or_else(|| &track.title).trim(),
                    track.duration_ms()
                )
            }
            StationExpectedState::TrackNarrationAfter {
                previous_track: _,
                narration,
            } => write!(
                f,
                "TrackNarrationAfter[{} ms]: {}",
                narration.as_ref().map_or(0, |f| { f.duration_ms() }),
                narration.as_ref().map_or("", |n| { &n.content }),
            ),
            StationExpectedState::TrackNarrationBefore {
                narration,
                imminent_track: _,
            } => {
                write!(
                    f,
                    "TrackNarrationBefore[{} ms]: {}",
                    narration.as_ref().map_or(0, |f| { f.duration_ms() }),
                    narration.as_ref().map_or("", |n| { &n.content }),
                )
            }
        }
    }
}

fn pick_from_vec<'a, I>(vec: &'a Vec<I>, rng: &mut Rand) -> Option<&'a I> {
    let max = vec.len() as u32;

    if max == 0 {
        return None;
    }

    let idx = rng.gen_range(0..max) as usize;
    return Some(&vec[idx]);
}

fn pick_next_track<'a>(
    tracks: &'a Vec<Track>,
    previous_tracks: &SlidingWindow<Track>,
    rng: &mut Rand,
) -> &'a Track {
    assert!(
        previous_tracks.capacity < tracks.len(),
        "Can't pick next track because the shuffle repetition mechanism window size is too large (it must be smaller than the track list). At some point, we wouldn't be able to pick a next track because all of them would have been played 'recently'."
    );

    loop {
        let picked_track = pick_from_vec(&tracks, rng).expect("Can't pick next track");
        if !previous_tracks.contents.contains(picked_track) {
            return picked_track;
        }
    }
}

struct SlidingWindow<I> {
    capacity: usize,
    contents: VecDeque<I>,
}

impl<I> SlidingWindow<I> {
    fn new(capacity: usize) -> SlidingWindow<I> {
        SlidingWindow {
            capacity,
            contents: VecDeque::with_capacity(capacity),
        }
    }

    fn append(&mut self, item: I) {
        while self.contents.len() >= self.capacity {
            self.contents.pop_front();
        }
        self.contents.push_back(item);
    }
}

pub fn determine_expected_current_state(station: &Station) -> (StationExpectedState, u64, u64) {
    let current_time_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let mut rng = Rand::with_seed(station.seed);
    let mut state = StationExpectedState::SilenceInterval;
    let mut elapsed = current_time_unix - STATION_EPOCH;
    let mut previous_tracks = SlidingWindow::<Track>::new(8);
    let mut step_duration: u64;

    loop {
        let next_state: StationExpectedState;

        // println!("{}", state);

        match &state {
            StationExpectedState::TrackNarrationBefore {
                narration,
                imminent_track,
            } => {
                if narration.is_none() {
                    step_duration = 0;
                } else {
                    step_duration = narration.as_ref().unwrap().duration_ms();
                }
                next_state = StationExpectedState::Track {
                    track: imminent_track.clone(),
                }
            }
            StationExpectedState::Track { track } => {
                step_duration = track.duration_ms();

                // pick random ending narration if any
                let narration = pick_from_vec(&track.narration_after, &mut rng);
                next_state = StationExpectedState::TrackNarrationAfter {
                    previous_track: track.clone(),
                    narration: narration.cloned(),
                };
            }
            StationExpectedState::TrackNarrationAfter {
                previous_track,
                narration,
            } => {
                if narration.is_none() {
                    step_duration = 0;
                } else {
                    step_duration = narration.as_ref().unwrap().duration_ms();
                }
                previous_tracks.append(previous_track.clone());
                next_state = StationExpectedState::SilenceInterval {};
            }
            StationExpectedState::SilenceInterval => {
                step_duration = SILENCE_INTERVAL_MS;

                let next_track = pick_next_track(&station.tracks, &previous_tracks, &mut rng);
                let next_track_narration_before =
                    pick_from_vec(&next_track.narration_before, &mut rng);
                next_state = StationExpectedState::TrackNarrationBefore {
                    narration: next_track_narration_before.cloned(),
                    imminent_track: next_track.clone(),
                }
            }
        }

        // println!(
        //     "Elapsed: {}, current step duration: {}",
        //     elapsed, step_duration
        // );

        if step_duration < elapsed {
            // then this step runs to completion
            state = next_state;
            elapsed -= step_duration;
        } else {
            // then this current step is not completed yet, we've found the current state
            break;
        }
    }

    println!(
        "Current state: {}, elapsed for this step: {} / {} ms",
        state, elapsed, step_duration
    );

    return (state, elapsed, step_duration);
}
