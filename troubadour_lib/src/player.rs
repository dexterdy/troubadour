#![allow(dead_code)]

use rodio::{
    source::{Buffered, Zero},
    Decoder, OutputStream, OutputStreamHandle, Sink, Source,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::error::{convert_read_file_error, Error, ErrorVariant, FileKind};

#[derive(Serialize, Deserialize)]
pub(crate) struct Serializable {
    media: PathBuf,
    name: String,
    group: Option<String>,
    volume: u32,
    looping: bool,
    loop_gap: Duration,
    delay_length: Duration,
    cut_end: Duration,
    cut_start: Duration,
}

struct Audio {
    stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Sink,
    source: Buffered<Decoder<File>>,
}

impl Audio {
    fn new(media: &PathBuf) -> Result<Self, Error> {
        let (stream, handle) = OutputStream::try_default().map_err(|e| Error {
            msg: "error: failed to set up your audio device.".to_string(),
            variant: ErrorVariant::AudioDeviceSetupFailed,
            source: Some(e.into()),
        })?;
        let sink = Sink::try_new(&handle).map_err(|e| Error {
            msg: "error: failed to set up your audio device.".to_string(),
            variant: ErrorVariant::AudioDeviceSetupFailed,
            source: Some(e.into()),
        })?;

        let file = File::open(&media)
            .map_err(|err| convert_read_file_error(&media, err, FileKind::Media))?;
        let source = Decoder::new(file)
            .map_err(|e| {
                Error {
                    msg: "error: cannot play file. The format might not be supported, or the data is corrupt."
                        .to_string(),
                    variant: ErrorVariant::DecoderFailed,
                    source: Some(e.into()),
                }
            })?
            .buffered();

        Ok(Self {
            stream,
            handle,
            sink,
            source,
        })
    }
}

pub struct Player {
    audio: Audio,
    media: PathBuf,
    last_time_poll: Option<Instant>,
    time_at_last_poll: Duration,
    pub name: String,
    pub base_length: Duration,
    pub group: Option<String>,
    playing: bool,
    paused: bool,
    pub volume: u32,
    pub looping: bool,
    pub loop_gap: Duration,
    pub delay_length: Duration,
    pub cut_end: Duration,
    pub cut_start: Duration,
}

macro_rules! optional {
    ($cond:expr, {$($rule:stmt);+;}, {$($alt_rule:stmt);+;}, $rest:expr) => {
        if $cond {
            $($rule)+
            $rest
        } else {
            $($alt_rule)+
            $rest
        }
    };
    ($cond:expr, {$($rule:stmt);+;}, $rest:expr) => {
        optional!($cond, {$($rule);+;}, {();}, $rest)
    };
    ($cond:expr, $rule:stmt, $rest:expr) => {
        optional!($cond, {$rule;}, {();}, $rest)
    };
    ($cond:expr, $rule:stmt, $alt_rule:stmt, $rest:expr) => {
        optional($cond, {$rule;}, {$alt_rule;}, $rest)
    }
}

impl Player {
    pub fn new(media: PathBuf, name: String) -> Result<Self, Error> {
        let audio = Audio::new(&media)?;
        let length = audio.source.total_duration().unwrap();

        Ok(Self {
            audio,
            media,
            last_time_poll: None,
            time_at_last_poll: Duration::from_secs(0),
            name,
            base_length: length,
            group: None,
            playing: false,
            paused: false,
            volume: 100,
            looping: false,
            loop_gap: Duration::from_secs(0),
            delay_length: Duration::from_secs(0),
            cut_end: Duration::from_secs(0),
            cut_start: Duration::from_secs(0),
        })
    }

    pub fn copy(&self, new_name: &str) -> Result<Self, Error> {
        let audio = Audio::new(&self.media)?;
        let length = audio.source.total_duration().unwrap();

        Ok(Self {
            audio,
            media: self.media.clone(),
            last_time_poll: None,
            time_at_last_poll: Duration::from_secs(0),
            name: new_name.to_string(),
            base_length: length,
            group: self.group.clone(),
            playing: self.playing.clone(),
            paused: self.paused.clone(),
            volume: self.volume.clone(),
            looping: self.looping.clone(),
            loop_gap: self.loop_gap.clone(),
            delay_length: self.delay_length.clone(),
            cut_end: self.cut_end.clone(),
            cut_start: self.cut_start.clone(),
        })
    }

    pub(crate) fn to_serializable(&self) -> Serializable {
        Serializable {
            name: self.name.clone(),
            group: self.group.clone(),
            media: self.media.clone(),
            volume: self.volume,
            looping: self.looping,
            loop_gap: self.loop_gap,
            delay_length: self.delay_length,
            cut_end: self.cut_end,
            cut_start: self.cut_start,
        }
    }

    pub(crate) fn from_serializable(player: &Serializable) -> Result<Self, Error> {
        let audio = Audio::new(&player.media)?;
        let length = audio.source.total_duration().unwrap();

        let mut new_player = Self {
            audio,
            media: player.media.clone(),
            last_time_poll: None,
            time_at_last_poll: Duration::from_secs(0),
            name: player.name.clone(),
            base_length: length,
            group: player.group.clone(),
            playing: false,
            paused: false,
            volume: player.volume,
            looping: player.looping,
            loop_gap: player.loop_gap,
            delay_length: player.delay_length,
            cut_end: player.cut_end,
            cut_start: player.cut_start,
        };
        new_player.volume(player.volume);
        Ok(new_player)
    }

    pub fn play(&mut self) -> Result<(), Error> {
        if self.get_is_playing() {
            return Ok(());
        }
        self.last_time_poll = Some(Instant::now());
        if self.get_is_paused() {
            self.audio.sink.play();
        } else {
            self.time_at_last_poll = Duration::from_secs(0);
            self.apply_settings(true, Duration::from_secs(0))?;
        }
        self.playing = true;
        self.paused = false;
        Ok(())
    }

    pub fn pause(&mut self) {
        if self.get_is_playing() {
            self.time_at_last_poll = self.get_play_time();
            self.last_time_poll = Some(Instant::now());
            self.audio.sink.pause();
            self.paused = true;
            self.playing = false;
        }
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.paused = false;
        self.last_time_poll = None;
        self.time_at_last_poll = Duration::from_secs(0);
        self.audio.sink.clear();
    }

    pub fn volume(&mut self, volume: u32) {
        self.volume = volume;
        let real_volume = f32::powf(
            2.0,
            f32::sqrt(f32::sqrt(f32::sqrt(volume as f32 / 100.0))).mul_add(192.0, -192.0) / 6.0,
        );
        self.audio.sink.set_volume(real_volume);
    }

    // The functions `set_delay`, `cut_start`, `cut_end` and `toggle_loop` can modify
    // starting and ending points of the loop/sound. They include logic to adjust the
    // play head so that it lines up with where the player left off before the adjustment

    pub fn set_delay(&mut self, delay: Duration) -> Result<(), Error> {
        let mut start_at = Duration::from_secs(0);
        if self.last_time_poll.is_some() {
            let play_time = self.get_looped_play_time();
            if let Some(play_time) = play_time {
                // past prev delay. Add own delay
                start_at = play_time + delay;
            } else {
                // not past delay. Prevent overshooting new delay
                start_at = self.get_play_time().min(delay);
            }
        }
        self.delay_length = delay;
        self.apply_settings(false, start_at)
    }

    pub fn cut_start(&mut self, cut: Duration) -> Result<(), Error> {
        let mut start_at = Duration::from_secs(0);
        if self.last_time_poll.is_some() {
            let play_time = self.get_looped_play_time();
            if let Some(play_time) = play_time {
                if play_time < cut {
                    // play head inside cut. move play head to new start of sound
                    start_at = cut + self.delay_length
                } else {
                    // adjust play head to line up with new time marks
                    start_at = (play_time - (cut - self.cut_start)) + self.delay_length;
                }
            } else {
                // not past delay
                start_at = self.get_play_time();
            }
        }
        self.cut_start = cut;
        self.apply_settings(false, start_at)
    }

    pub fn cut_end(&mut self, cut: Duration) -> Result<(), Error> {
        let mut start_at = Duration::from_secs(0);
        if self.last_time_poll.is_some() {
            let play_time = self.get_looped_play_time();
            if let Some(play_time) = play_time {
                let cut_location = self.base_length - cut - self.cut_start;
                let end_location = self.base_length - self.cut_end - self.cut_start;
                if cut_location < end_location
                    && play_time > cut_location
                    && play_time < end_location
                {
                    // play head in cut. move play head to new end of sound
                    start_at = cut_location + self.delay_length
                } else if cut_location > end_location && play_time > end_location {
                    // play head after cut, adjust play head to line up with new time marks
                    start_at = play_time + self.delay_length + (cut_location - end_location);
                } else {
                    // play head before cut, nothing to adjust
                    start_at = play_time + self.delay_length
                }
            } else {
                // not past delay
                start_at = self.get_play_time();
            }
        }
        self.cut_end = cut;
        self.apply_settings(false, start_at)
    }

    pub fn toggle_loop(&mut self, looping: bool, length: Duration) -> Result<(), Error> {
        let mut start_at = Duration::from_secs(0);
        if self.last_time_poll.is_some() {
            let play_time = self.get_looped_play_time();
            if let Some(play_time) = play_time {
                // prevent overshooting new loop gap
                start_at = (play_time + self.delay_length).min(
                    self.get_loop_length()
                        + (if looping {
                            length
                        } else {
                            Duration::from_secs(0)
                        } - self.loop_gap),
                );
            } else {
                // not past delay
                start_at = self.get_play_time();
            }
        }
        self.looping = looping;
        self.loop_gap = length;
        self.apply_settings(false, start_at)
    }

    pub fn get_play_time(&self) -> Duration {
        if self.get_is_playing() && self.last_time_poll.is_some() {
            self.time_at_last_poll + self.last_time_poll.unwrap().elapsed()
        } else if !self.get_is_playing() && self.get_is_paused() {
            self.time_at_last_poll
        } else {
            Duration::from_secs(0)
        }
    }

    pub fn get_loop_length(&self) -> Duration {
        self.base_length - (self.cut_start + self.cut_end) + self.loop_gap
    }

    /// Calculates the remainder of play time divided by loop length.
    /// Will return `None` when the player is stopped or still in delay.
    pub fn get_looped_play_time(&self) -> Option<Duration> {
        let play_time = self.get_play_time();
        if play_time > self.delay_length {
            Some(duration_rem(
                self.get_play_time() - self.delay_length,
                self.get_loop_length(),
            ))
        } else {
            None
        }
    }

    /// Checks whether the player is paused, based on the player and sink states.
    pub fn get_is_paused(&self) -> bool {
        self.paused && !self.audio.sink.empty() && !self.playing && self.audio.sink.is_paused()
    }

    /// Checks whether the player is playing, based on the player and sink states.
    pub fn get_is_playing(&self) -> bool {
        self.playing && !self.audio.sink.empty() && !self.paused && !self.audio.sink.is_paused()
    }

    fn apply_settings(&self, play_if_not_playing: bool, start_at: Duration) -> Result<(), Error> {
        let was_playing = self.get_is_playing();

        let audio = &self.audio;
        if !audio.sink.empty() {
            audio.sink.skip_one();
        }

        let decoder = self.audio.source.clone();

        optional!(
            self.cut_end > Duration::from_secs(0),
            let decoder = decoder.take_duration(self.base_length - self.cut_end),
        optional!(
            self.cut_start > Duration::from_secs(0),
            let decoder = decoder.skip_duration(self.cut_start),
        optional!(
            self.looping && self.loop_gap > Duration::from_secs(0),
            let decoder = {
                let to_take = decoder.total_duration().unwrap() + self.loop_gap;
                let silence: Zero<i16> = Zero::new(decoder.channels(), decoder.sample_rate());
                let decoder_padded = decoder.mix(silence);
                decoder_padded.take_duration(to_take)
            },
        optional!(
            self.looping,
            let decoder = decoder.repeat_infinite(),
        optional!(
            self.delay_length > Duration::from_secs(0),
            let decoder = decoder.delay(self.delay_length),
        optional!(
            start_at > Duration::from_secs(0),
            let decoder = decoder.skip_duration(start_at),
        audio.sink.append(decoder)
        ))))));

        if was_playing || play_if_not_playing {
            audio.sink.play();
        } else {
            audio.sink.pause();
        }

        Ok(())
    }
}

fn duration_rem(a: Duration, b: Duration) -> Duration {
    Duration::from_secs_f64(a.as_secs_f64() % b.as_secs_f64())
}

#[test]
fn player_functionality() {
    let mut player = Player::new(
        PathBuf::from(r"/home/jphagedoorn/Downloads/Marimba name that tune no 1 [Q5mpenYcXyw].mp3"),
        "giant".to_string(),
    )
    .unwrap();
    println!("delay");
    player.set_delay(Duration::from_secs(3)).unwrap();
    println!("play");
    player.play().unwrap();
    std::thread::sleep(Duration::from_secs(6));
    println!("apply settings test");
    player.set_delay(Duration::from_secs(0)).unwrap();
    std::thread::sleep(Duration::from_secs(3));
    println!("stop");
    player.stop();
    std::thread::sleep(Duration::from_secs(1));
    println!("play");
    player.play().unwrap();
    std::thread::sleep(Duration::from_secs(3));
    println!("toggle loop");
    player.cut_start(Duration::from_secs(5)).unwrap();
    player.toggle_loop(true, Duration::from_secs(5)).unwrap();
    player.play().unwrap();
    std::thread::sleep(Duration::from_secs_f32(17.5));
    println!("pause");
    player.pause();
    std::thread::sleep(Duration::from_secs(1));
    println!("play");
    player.play().unwrap();
    std::thread::sleep(Duration::from_secs(3));
}

#[test]
fn duration_rem_test() {
    let play_time = Duration::from_secs_f64(23.0);
    let length = Duration::from_secs_f64(12.0);
    let looped_time = duration_rem(play_time, length);
    assert_eq!(looped_time, Duration::from_secs(11))
}
