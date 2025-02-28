#![allow(dead_code)]

use paste::item;
use rodio::{source::Zero, Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    fs::File,
    io::BufReader,
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::error::{convert_read_file_error, Error, ErrorVariant, FileKind};

#[derive(Serialize, Deserialize)]
pub struct Serializable {
    media: PathBuf,
    name: String,
    group: Option<String>,
    volume: u32,
    looping: bool,
    loop_length: Option<Duration>,
    delay_length: Duration,
    take_length: Option<Duration>,
    skip_length: Duration,
}

pub struct Player {
    stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Sink,
    media: PathBuf,
    file_handle: RefCell<File>,
    last_time_poll: Option<Instant>,
    time_at_last_poll: Duration,
    pub name: String,
    pub group: Option<String>,
    pub playing: bool,
    pub paused: bool,
    pub volume: u32,
    pub looping: bool,
    pub loop_length: Option<Duration>,
    pub delay_length: Duration,
    pub take_length: Option<Duration>,
    pub skip_length: Duration,
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

macro_rules! as_builder {
    ($($v:vis fn $NAME:ident (&mut $self:ident, $($name:ident:$args:ty),*) $body:block)+) => {
        $($v fn $NAME (&mut $self, $($name:$args),*) {
            $body
        }
        item! {
            $v fn [<$NAME _and>] (mut self, $($name:$args),*) -> Self{
                self.$NAME($($name),*);
                self
            }
        })+
    };
}

fn get_device_stuff() -> Result<(OutputStream, OutputStreamHandle, Sink), Error> {
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
    Ok((stream, handle, sink))
}

impl Player {
    pub fn new(media: PathBuf, name: String) -> Result<Self, Error> {
        let (stream, handle, sink) = get_device_stuff()?;
        let file = File::open(&media)
            .map_err(|err| convert_read_file_error(&media, err, FileKind::Media))?;
        Ok(Self {
            name,
            group: None,
            media,
            file_handle: RefCell::new(file),
            playing: false,
            paused: false,
            volume: 100,
            looping: false,
            loop_length: None,
            delay_length: Duration::from_secs(0),
            take_length: None,
            skip_length: Duration::from_secs(0),
            stream,
            handle,
            sink,
            last_time_poll: None,
            time_at_last_poll: Duration::from_secs(0),
        })
    }

    pub fn to_serializable(&self) -> Serializable {
        Serializable {
            name: self.name.clone(),
            group: self.group.clone(),
            media: self.media.clone(),
            volume: self.volume,
            looping: self.looping,
            loop_length: self.loop_length,
            delay_length: self.delay_length,
            take_length: self.take_length,
            skip_length: self.skip_length,
        }
    }

    pub fn from_serializable(player: &Serializable) -> Result<Self, Error> {
        let (stream, handle, sink) = get_device_stuff()?;
        let file = File::open(&player.media)
            .map_err(|err| convert_read_file_error(&player.media, err, FileKind::Media))?;
        let mut new_player = Self {
            name: player.name.clone(),
            group: player.group.clone(),
            media: player.media.clone(),
            file_handle: RefCell::new(file),
            playing: false,
            paused: false,
            volume: player.volume,
            looping: player.looping,
            loop_length: player.loop_length,
            delay_length: player.delay_length,
            take_length: player.take_length,
            skip_length: player.skip_length,
            stream,
            handle,
            sink,
            last_time_poll: None,
            time_at_last_poll: Duration::from_secs(0),
        };
        new_player.volume(player.volume);
        Ok(new_player)
    }

    as_builder! {
        pub fn set_delay(&mut self, delay: Duration) {
            self.delay_length = delay;
        }

        pub fn skip_duration(&mut self, skip: Duration) {
            self.skip_length = skip;
        }

        pub fn take_duration(&mut self, take: Option<Duration>) {
            self.take_length = take;
        }

        pub fn toggle_loop(&mut self, looping: bool) {
            self.looping = looping;
        }

        pub fn loop_length(&mut self, length: Option<Duration>){
            self.loop_length = length;
        }
    }

    fn apply_settings_internal(
        &self,
        start_immediately: bool,
        start_at: Duration,
    ) -> Result<(), Error> {
        // possible edge case: prev buffer reads from file at same time as this operation, causing a race condition?
        let is_empty = self.sink.empty();
        let file = File::open(&self.media)
            .map_err(|err| convert_read_file_error(&self.media, err, FileKind::Media))?;
        self.file_handle.replace(file);
        let media = BufReader::new(
            self.file_handle
                .borrow()
                .try_clone()
                .map_err(|err| convert_read_file_error(&self.media, err, FileKind::Media))?,
        );
        let decoder = Decoder::new(media).map_err(|e| Error {
            msg:"error: cannot play file. The format might not be supported, or the data is corrupt.".to_string(), 
            variant: ErrorVariant::DecoderFailed,
            source: Some(e.into()),
        })?;

        optional!(
            self.take_length.is_some() && self.take_length.unwrap() > Duration::from_secs(0) && (
                !self.looping || self.loop_length.is_none() || (
                    self.loop_length.is_some() &&
                    self.take_length.unwrap() < self.loop_length.unwrap()
                )
            ),
            let decoder = decoder.take_duration(self.take_length.unwrap()),
        optional!(
            self.skip_length > Duration::from_secs(0),
            let decoder = decoder.skip_duration(self.skip_length),
        optional!(
            self.looping && self.loop_length.is_some(),
            let decoder = {
                let silence: Zero<i16> = Zero::new(decoder.channels(), decoder.sample_rate());
                let decoder_padded = decoder.mix(silence);
                decoder_padded.take_duration(self.loop_length.unwrap())
            },
        optional!(
            self.looping,
            let decoder = {decoder.repeat_infinite()},
        optional!(start_at > self.skip_length,
            let decoder = decoder.skip_duration(start_at - self.skip_length),
        optional!(
            self.delay_length > Duration::from_secs(0),
            let decoder = decoder.delay(self.delay_length),
        self.sink.append(decoder)
        ))))));

        if !is_empty {
            self.sink.skip_one();
        }
        if start_immediately {
            self.sink.play();
        } else {
            self.sink.pause();
        }
        Ok(())
    }

    pub fn apply_settings(self, play_if_not_playing: bool) -> Result<Self, Error> {
        self.apply_settings_in_place(play_if_not_playing)?;
        Ok(self)
    }

    pub fn apply_settings_in_place(&self, play_if_not_playing: bool) -> Result<(), Error> {
        let play_time = self.get_play_time();
        self.apply_settings_internal(self.get_is_playing() || play_if_not_playing, play_time)
    }

    //TODO: an implementation of get_play_time() which relies on the play data, instead of the time crate
    pub fn get_play_time(&self) -> Duration {
        if self.get_is_playing() && self.last_time_poll.is_some() {
            self.time_at_last_poll + self.last_time_poll.unwrap().elapsed()
        } else if !self.get_is_playing() && self.get_is_paused() {
            self.time_at_last_poll
        } else {
            Duration::from_secs(0)
        }
    }

    pub fn get_is_paused(&self) -> bool {
        self.paused && !self.sink.empty() && !self.playing && self.sink.is_paused()
    }

    pub fn get_is_playing(&self) -> bool {
        self.playing && !self.sink.empty() && !self.paused && !self.sink.is_paused()
    }

    pub fn play(&mut self) -> Result<(), Error> {
        if self.get_is_playing() {
            return Err(Error {
                msg: format!("error: player {} is already playing", self.name),
                variant: ErrorVariant::OperationFailed,
                source: None,
            });
        }
        if self.get_is_paused() {
            self.sink.play();
        } else {
            self.time_at_last_poll = Duration::from_secs(0);
            self.apply_settings_in_place(true)?;
        }
        self.last_time_poll = Some(Instant::now());
        self.playing = true;
        self.paused = false;
        Ok(())
    }

    pub fn pause(&mut self) {
        if self.get_is_playing() {
            self.time_at_last_poll = self.get_play_time();
            self.last_time_poll = Some(Instant::now());
            self.sink.pause();
            self.paused = true;
            self.playing = false;
        }
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.paused = false;
        self.last_time_poll = None;
        self.time_at_last_poll = Duration::from_secs(0);
        self.sink.clear();
    }

    pub fn volume(&mut self, volume: u32) {
        self.volume = volume;
        let real_volume = f32::powf(
            2.0,
            f32::sqrt(f32::sqrt(f32::sqrt(volume as f32 / 100.0))).mul_add(192.0, -192.0) / 6.0,
        );
        self.sink.set_volume(real_volume);
    }
}

#[test]
fn player_functionality() {
    let mut player = Player::new(
        PathBuf::from(r"/home/jphagedoorn/Downloads/Marimba name that tune no 1 [Q5mpenYcXyw].mp3"),
        "giant".to_string(),
    )
    .unwrap();
    println!("delay");
    player.set_delay(Duration::from_secs(3));
    println!("play");
    player.play().unwrap();
    std::thread::sleep(Duration::from_secs(6));
    println!("apply settings test");
    player = player
        .set_delay_and(Duration::from_secs(0))
        .apply_settings(false)
        .unwrap();
    std::thread::sleep(Duration::from_secs(3));
    println!("stop");
    player.stop();
    std::thread::sleep(Duration::from_secs(1));
    println!("play");
    player.play().unwrap();
    std::thread::sleep(Duration::from_secs(3));
    println!("toggle loop");
    player = player
        .skip_duration_and(Duration::from_secs(5))
        .loop_length_and(Some(Duration::from_secs(15)))
        .toggle_loop_and(true)
        .apply_settings(false)
        .unwrap();
    player.play().unwrap();
    std::thread::sleep(Duration::from_secs_f32(17.5));
    println!("pause");
    player.pause();
    std::thread::sleep(Duration::from_secs(1));
    println!("play");
    player.play().unwrap();
    std::thread::sleep(Duration::from_secs(3));
}
