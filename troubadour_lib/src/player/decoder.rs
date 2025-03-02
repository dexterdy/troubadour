use std::{
    io::{Read, Seek},
    sync::{Arc, RwLock},
};

use rodio::{decoder::DecoderError, Source};

pub struct Decoder<R: Read + Seek> {
    inner: Arc<RwLock<rodio::Decoder<R>>>,
}

impl<R: Read + Seek + Send + Sync + 'static> Decoder<R> {
    pub fn new(r: R) -> Result<Self, DecoderError> {
        Ok(Self {
            inner: Arc::new(RwLock::new(rodio::Decoder::new(r)?)),
        })
    }
}

impl<R: Read + Seek> Iterator for Decoder<R> {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.write().ok().and_then(|mut i| i.next())
    }
}

impl<R: Read + Seek> Source for Decoder<R> {
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.read().unwrap().current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.inner.read().unwrap().channels()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.read().unwrap().sample_rate()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.inner.read().unwrap().total_duration()
    }
}

impl<R: Read + Seek> Clone for Decoder<R> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
