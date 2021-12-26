use crate::{handler::Handler, DecibelResponder};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Host, Stream,
};
use eyre::Result;

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

    pub fn stop(&mut self) {
        if let State::Running(stream) = &self.state {
            let _ = stream.pause();
        }

        self.state = State::Stopped;
    }

    pub fn run(&mut self, responder: Box<dyn DecibelResponder>) -> Result<()> {
        let mut state = State::Init;
        std::mem::swap(&mut self.state, &mut state);

        match state {
            State::Init | State::Stopped => {
                self.state = self.init_handler();
                self.run(responder)?;
                Ok(())
            }
            State::Ready(handler) => {
                let stream = handler.run(responder)?;
                self.state = State::Running(stream);
                Ok(())
            }
            State::Running(stream) => {
                self.state = State::Running(stream);
                Ok(())
            }
        }
    }

    fn init_handler(&self) -> State {
        let device = self
            .host
            .default_input_device()
            .expect("no input device available");

        let config = device
            .default_input_config()
            .expect("Failed to get default input config");

        let handler = Handler::new(device, config);
        State::Ready(handler)
    }
}
