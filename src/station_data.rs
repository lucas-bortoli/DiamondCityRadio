use std::{
    fs,
    os::windows::fs::MetadataExt,
    time::{SystemTime, UNIX_EPOCH},
};

use frand::Rand;

pub const BIT_DEPTH: u32 = 16;
pub const SAMPLE_RATE: u32 = 44100;
pub const CHANNEL_COUNT: u32 = 2;
pub const STATION_EPOCH: u64 = 10000;
pub const STATION_TRACKS_RNG_SEED: u64 = 2003;

#[derive(Debug)]
pub struct Track {
    pub title: String,
    pub source: String,
    pub size_bytes: u64,
}

impl Track {
    /// Retorna a duração dessa track em segundos
    pub fn duration_s(&self) -> f64 {
        let bytes_per_second = SAMPLE_RATE * (BIT_DEPTH / 8) * CHANNEL_COUNT;
        return (self.size_bytes - 44) as f64 / bytes_per_second as f64;
    }

    pub fn duration_ms(&self) -> u64 {
        let bytes_per_millisecond = (SAMPLE_RATE / 1000) * (BIT_DEPTH / 8) * CHANNEL_COUNT;
        return (self.size_bytes - 44) / bytes_per_millisecond as u64;
    }

    pub fn time_to_byte_offset(&self, time_ms: u64) -> u64 {
        let bytes_per_millisecond = (SAMPLE_RATE / 1000) * (BIT_DEPTH / 8) * CHANNEL_COUNT;

        return 44 + time_ms * bytes_per_millisecond as u64;
    }
}

#[derive(Debug)]
pub struct Station {
    pub title: String,
    pub tracks: Vec<Track>,
}

#[derive(Debug)]
pub struct ResumeInformation<'a> {
    pub track: &'a Track,
    pub seek_ms: u64,
}

impl Station {
    pub fn new() -> Station {
        let mut station = Station {
            title: String::from("Diamond City Radio"),
            tracks: vec![
                Track {
                    title: String::from("Anything Goes by Cole Porter (1934)"),
                    source: String::from("./diamond_city_radio/anything_goes.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("Atom Bomb Baby by The Five Stars (1957)"),
                    source: String::from("./diamond_city_radio/atom_bomb_baby.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("Baby It's Just You* by Lynda Carter (2015)"),
                    source: String::from("./diamond_city_radio/baby_its_just_you.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("Butcher Pete (Part 1) by Roy Brown (1950)"),
                    source: String::from("./diamond_city_radio/butcher_pete_part_1.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("Butcher Pete (Part 2) by Roy Brown (1950)"),
                    source: String::from("./diamond_city_radio/butcher_pete_part_2.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from(
                        "Civilization, also called \"Bongo Bongo Bongo,\" by Vic Schoen and his Orchestra Danny Kaye and The Andrews Sisters (1947)",
                    ),
                    source: String::from("./diamond_city_radio/civilization_bongo_bongo_bongo.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("Crawl Out Through the Fallout by Sheldon Allman (1960)"),
                    source: String::from("./diamond_city_radio/crawl_out_through_the_fallout.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("Crazy He Calls Me by Billie Holiday (1949)"),
                    source: String::from("./diamond_city_radio/crazy_he_calls_me.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from(
                        "Dear Hearts and Gentle People by Bob Crosby & The Bob Cats (1949)",
                    ),
                    source: String::from("./diamond_city_radio/dear_hearts_and_gentle_people.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from(
                        "Easy Living by Billie Holiday with Teddy Wilson and his orchestra (1937)",
                    ),
                    source: String::from("./diamond_city_radio/easy_living.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("Good Neighbor* by Lynda Carter (2015)"),
                    source: String::from("./diamond_city_radio/good_neighbor.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from(
                        "Good Rocking Tonight by Roy Brown with Bob Ogden and his orchestra (1947)",
                    ),
                    source: String::from("./diamond_city_radio/good_rocking_tonight.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("Grandma Plays the Numbers by Wynonie Harris (1949)"),
                    source: String::from("./diamond_city_radio/grandma_plays_the_numbers.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("Happy Times by Bob Crosby (1949)"),
                    source: String::from("./diamond_city_radio/happy_times.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from(
                        "He's a Demon, He's a Devil, He's a Doll by Betty Hutton (1950)",
                    ),
                    source: String::from(
                        "./diamond_city_radio/hes_a_demon_hes_a_devil_hes_a_doll.wav",
                    ),
                    size_bytes: 0,
                },
                Track {
                    title: String::from(
                        "I Don't Want to Set the World on Fire by The Ink Spots (1941)",
                    ),
                    source: String::from(
                        "./diamond_city_radio/i_dont_want_to_set_the_world_on_fire.wav",
                    ),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("I'm the One You're Looking For* by Lynda Carter (2015)"),
                    source: String::from("./diamond_city_radio/im_the_one_youre_looking_for.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from(
                        "Into Each Life Some Rain Must Fall by Ella Fitzgerald and The Ink Spots (1944)",
                    ),
                    source: String::from(
                        "./diamond_city_radio/into_each_life_some_rain_must_fall.wav",
                    ),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("It's a Man by Betty Hutton (1951)"),
                    source: String::from("./diamond_city_radio/its_a_man.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("It's All Over But the Crying by The Ink Spots (1947)"),
                    source: String::from("./diamond_city_radio/its_all_over_but_the_crying.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("Keep A Knockin by Louis Jordan (1939)"),
                    source: String::from("./diamond_city_radio/keep_a_knockin.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("Man Enough* by Lynda Carter (2015)"),
                    source: String::from("./diamond_city_radio/man_enough.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("Maybe by The Ink Spots (1940)"),
                    source: String::from("./diamond_city_radio/maybe.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from(
                        "Mighty, Mighty Man by Earl Barnes and his Orchestra, featuring Roy Brown (1948)",
                    ),
                    source: String::from("./diamond_city_radio/mighty_mighty_man.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from(
                        "One More Tomorrow by Marjorie Hughes, featuring Frankie Carle and his Orchestra (1946)",
                    ),
                    source: String::from("./diamond_city_radio/one_more_tomorrow.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from(
                        "Orange Colored Sky by Stan Kenton, featuring Nat King Cole (1950)",
                    ),
                    source: String::from("./diamond_city_radio/orange_colored_sky.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from(
                        "Personality by The Pied Pipers, featuring Johnny Mercer (1946)",
                    ),
                    source: String::from("./diamond_city_radio/personality.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from(
                        "Pistol Packin' Mama by Vic Schoen and his Orchestra, featuring Bing Crosby and The Andrews Sisters (1943)",
                    ),
                    source: String::from("./diamond_city_radio/pistol_packin_mama.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("Right Behind You Baby by Ray Smith (1958)"),
                    source: String::from("./diamond_city_radio/right_behind_you_baby.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from(
                        "Rocket 69 by Todd Rhodes and His Toddlers, featuring Connie Allen (1951)",
                    ),
                    source: String::from("./diamond_city_radio/rocket_69.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("Sixty Minute Man by Billy Ward and his Dominoes (1951)"),
                    source: String::from("./diamond_city_radio/sixty_minute_man.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("The End of the World by Skeeter Davis (1962)"),
                    source: String::from("./diamond_city_radio/the_end_of_the_world.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("The Wanderer by Dion (1961)"),
                    source: String::from("./diamond_city_radio/the_wanderer.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("Train Train* by Lynda Carter (2015)"),
                    source: String::from("./diamond_city_radio/train_train.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from(
                        "Undecided by Chick Webb and his Orchestra, featuring Ella Fitzgerald (1938)",
                    ),
                    source: String::from("./diamond_city_radio/undecided.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("Uranium Fever by Elton Britt (1955)"),
                    source: String::from("./diamond_city_radio/uranium_fever.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("Uranium Rock by Warren Smith (1958)"),
                    source: String::from("./diamond_city_radio/uranium_rock.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("Way Back Home by Bob Crosby and The Bob Cats (1950)"),
                    source: String::from("./diamond_city_radio/way_back_home.wav"),
                    size_bytes: 0,
                },
                Track {
                    title: String::from("Whole Lotta Shakin' Goin' On by Big Maybelle (1955)"),
                    source: String::from("./diamond_city_radio/whole_lotta_shakin_goin_on.wav"),
                    size_bytes: 0,
                },
            ],
        };

        for track in &mut station.tracks {
            let metadata = fs::metadata(&track.source).expect("Failed to query file data");
            track.size_bytes = metadata.file_size();
        }

        return station;
    }

    pub fn determine_current_track_for_resuming(&self) -> ResumeInformation {
        let mut track_rng = Rand::with_seed(STATION_TRACKS_RNG_SEED);
        let current_time_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("O tempo voltou para trás da timestamp UNIX")
            .as_millis() as u64;

        fn next_track<'a>(station: &'a Station, track_rng: &mut Rand) -> &'a Track {
            assert!(station.tracks.len() >= 1, "Não há trilhas nessa estação.");
            let track_index = track_rng.r#gen::<usize>() % station.tracks.len();
            return station.tracks.get(track_index).unwrap();
        }

        let mut current_track = next_track(&self, &mut track_rng);
        let mut current_track_time = current_time_unix - STATION_EPOCH;
        while current_track_time >= current_track.duration_ms() {
            current_track_time = current_track_time - current_track.duration_ms();
            current_track = next_track(&self, &mut track_rng);
        }

        return ResumeInformation {
            track: current_track,
            seek_ms: current_track_time,
        };
    }
}
