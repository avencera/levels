mod inner;

use cpal::{Device, Stream, SupportedStreamConfig};
use eyre::Result;

use crate::DecibelResponder;

use self::inner::InnerHandler;

pub enum Handler {
    F32(InnerHandler<f32>),
    I16(InnerHandler<i16>),
    U16(InnerHandler<u16>),
}

impl Handler {
    pub fn new(device: Device, config: SupportedStreamConfig) -> Self {
        match config.sample_format() {
            cpal::SampleFormat::F32 => Handler::F32(InnerHandler::new(device, config)),
            cpal::SampleFormat::I16 => Handler::I16(InnerHandler::new(device, config)),
            cpal::SampleFormat::U16 => Handler::U16(InnerHandler::new(device, config)),
        }
    }

    pub fn run(self, responder: Box<dyn DecibelResponder>) -> Result<Stream> {
        match self {
            Handler::F32(handler) => Ok(handler.run(responder)?),
            Handler::I16(handler) => Ok(handler.run(responder)?),
            Handler::U16(handler) => Ok(handler.run(responder)?),
        }
    }
}
