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
        self.inner.borrow_mut().next()
    }
}

impl<R: Read + Seek> Source for Decoder<R> {
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.borrow().current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.inner.borrow().channels()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.borrow().sample_rate()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.inner.borrow().total_duration()
    }
}

impl<R: Read + Seek> Clone for Decoder<R> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
