use std::{fs, os::windows::fs::MetadataExt};
use yaml_rust2::YamlLoader;

pub const BIT_DEPTH: u32 = 16;
pub const SAMPLE_RATE: u32 = 44100;
pub const CHANNEL_COUNT: u32 = 1;
pub const STATION_EPOCH: u64 = 1741013572723;

pub const POLL_BUFFER_SIZE_BYTES: usize =
    (SAMPLE_RATE * CHANNEL_COUNT * (BIT_DEPTH / 8) / 10) as usize;

pub trait SoundFile {
    fn source_filename(&self) -> String;
    fn size_bytes(&self) -> u64;
    fn duration_ms(&self) -> u64 {
        let bytes_per_millisecond = (SAMPLE_RATE / 1000) * (BIT_DEPTH / 8) * CHANNEL_COUNT;
        return (self.size_bytes() - 44) / bytes_per_millisecond as u64;
    }
    fn time_to_byte_offset(&self, time_ms: u64) -> u64 {
        let bytes_per_millisecond = (SAMPLE_RATE / 1000) * (BIT_DEPTH / 8) * CHANNEL_COUNT;

        return 44 + time_ms * bytes_per_millisecond as u64;
    }
}

#[derive(Clone, Debug)]
pub struct Narration {
    pub content: String,
    pub source: String,
    pub size_bytes: u64,
}

impl Narration {
    fn from_yaml(
        narration_def: &yaml_rust2::Yaml,
    ) -> Result<Narration, Box<dyn std::error::Error>> {
        let source_path = format!(
            "./diamond_city_radio/narration/{}",
            narration_def["source"]
                .as_str()
                .ok_or("Missing narration source")?
        );
        let metadata = fs::metadata(&source_path)?;
        let size_bytes = metadata.file_size();

        Ok(Narration {
            content: narration_def["narration"]
                .as_str()
                .ok_or("Missing narration content")?
                .to_string(),
            source: narration_def["source"]
                .as_str()
                .ok_or("Missing narration source")?
                .to_string(),
            size_bytes,
        })
    }

    pub fn duration_ms(&self) -> u64 {
        let bytes_per_millisecond = (SAMPLE_RATE / 1000) * (BIT_DEPTH / 8) * CHANNEL_COUNT;
        return (self.size_bytes - 44) / bytes_per_millisecond as u64;
    }
}

impl SoundFile for Narration {
    fn source_filename(&self) -> String {
        "./diamond_city_radio/narration/".to_string() + &self.source
    }

    fn size_bytes(&self) -> u64 {
        self.size_bytes
    }
}

#[derive(Clone, Debug)]
pub struct Track {
    pub title: String,
    pub source: String,
    pub size_bytes: u64,
    pub narration_before: Vec<Narration>,
    pub narration_after: Vec<Narration>,
}

impl SoundFile for Track {
    fn source_filename(&self) -> String {
        "./diamond_city_radio/".to_string() + &self.source
    }

    fn size_bytes(&self) -> u64 {
        self.size_bytes
    }
}

impl Track {
    fn from_yaml(track_def: &yaml_rust2::Yaml) -> Result<Track, Box<dyn std::error::Error>> {
        let source_path = format!(
            "./diamond_city_radio/{}",
            track_def["source"].as_str().ok_or("Missing track source")?
        );
        let metadata = fs::metadata(&source_path)?;
        let size_bytes = metadata.file_size();

        let track = Track {
            title: track_def["title"]
                .as_str()
                .ok_or("Missing track title")?
                .to_string(),
            source: track_def["source"]
                .as_str()
                .ok_or("Missing track source")?
                .to_string(),
            size_bytes,
            narration_before: Self::parse_narrations(track_def, "before")?,
            narration_after: Self::parse_narrations(track_def, "after")?,
        };

        Ok(track)
    }

    fn parse_narrations(
        track_def: &yaml_rust2::Yaml,
        key: &str,
    ) -> Result<Vec<Narration>, Box<dyn std::error::Error>> {
        track_def[key]
            .as_vec()
            .ok_or(format!("Missing {}", key))?
            .iter()
            .map(|narration_def| Narration::from_yaml(narration_def))
            .collect()
    }

    pub fn duration_ms(&self) -> u64 {
        let bytes_per_millisecond = (SAMPLE_RATE / 1000) * (BIT_DEPTH / 8) * CHANNEL_COUNT;
        return (self.size_bytes - 44) / bytes_per_millisecond as u64;
    }
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
    }
}

#[derive(Clone, Debug)]
pub struct Station {
    pub title: String,
    pub seed: u64,
    pub tracks: Vec<Track>,
}

impl Station {
    pub fn from_file(yaml_file_path: &str) -> Result<Station, Box<dyn std::error::Error>> {
        let radio_contents = fs::read_to_string(yaml_file_path)?;
        let radio_docs = YamlLoader::load_from_str(&radio_contents)?;

        let station = Station {
            title: radio_docs[0]["title"]
                .as_str()
                .ok_or("Missing title")?
                .to_string(),
            seed: radio_docs[0]["seed"].as_i64().ok_or("Missing seed")? as u64,
            tracks: radio_docs[0]["tracks"]
                .as_vec()
                .ok_or("Missing tracks")?
                .iter()
                .map(|track_def| Track::from_yaml(track_def))
                .collect::<Result<Vec<_>, _>>()?,
        };

        Ok(station)
    }
}
