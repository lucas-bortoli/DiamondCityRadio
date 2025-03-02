use std::time::{SystemTime, UNIX_EPOCH};

use frand::Rand;

use crate::station_data::{Narration, STATION_EPOCH, Station, Track};

const SILENCE_INTERVAL_MS: u32 = 200;

enum StationExpectedState {
    TrackNarrationBefore {
        narration: Narration,
        imminent_track: Track,
    },
    Track {
        track: Track,
    },
    TrackNarrationAfter {
        related_track: Track,
        narration: Narration,
    },
    SilenceInterval,
}

pub fn determine_expected_current_state(station: &Station) -> (StationExpectedState, u64) {
    let current_time_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let mut rng = Rand::with_seed(station.seed);
    let mut state = StationExpectedState::SilenceInterval;
    let mut remaining_time = current_time_unix - STATION_EPOCH;

    loop {
        match &state {
            StationExpectedState::TrackNarrationBefore {
                narration,
                imminent_track,
            } => {}
            StationExpectedState::Track { track } => {}
            StationExpectedState::TrackNarrationAfter {
                related_track,
                narration,
            } => {}
            StationExpectedState::SilenceInterval => {}
        }

        todo!("Replay the radio station to synchronize the current state.");
    }

    return (state, remaining_time);
}
