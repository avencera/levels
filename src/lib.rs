uniffi_macros::include_scaffolding!("levels");

mod amp;
mod handler;
mod util;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Host, Stream,
};
use eyre::Result;
use handler::Handler;

const INTERVAL: u16 = 100;
const LATENCY: u16 = 50;

enum State {
    Init,
    Stopped,
    Ready(Handler),
    Running(Stream),
}

pub struct App {
    host: Host,
    state: State,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let host = cpal::default_host();

        Self {
            host,
            state: State::Init,
        }
    }

    pub fn stop(mut self) {
        if let State::Running(stream) = self.state {
            let _ = stream.pause();
        }

        self.state = State::Stopped;
    }

    pub fn run(mut self) -> Result<Self> {
        match self.state {
            State::Init | State::Stopped => {
                self.init_handler();
                Ok(self.run()?)
            }
            State::Ready(handler) => {
                let stream = handler.run()?;
                self.state = State::Running(stream);
                Ok(self)
            }
            State::Running(_) => Ok(self),
        }
    }

    fn init_handler(&mut self) {
        let device = self
            .host
            .default_input_device()
            .expect("no input device available");

        let config = device
            .default_input_config()
            .expect("Failed to get default input config");

        let handler = Handler::new(device, config);
        self.state = State::Ready(handler);
    }
}

include!(concat!(env!("OUT_DIR"), "/levels.uniffi.rs"));
