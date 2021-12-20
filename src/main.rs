mod handlers;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Host, Stream,
};
use eyre::Result;
use handlers::Handler;

const INTERVAL: u16 = 100;
const LATENCY: u16 = INTERVAL / 2;

enum State {
    Init,
    Stopped,
    Ready(Handler),
    Running(Stream),
}

struct App {
    host: Host,
    state: State,
}

impl App {
    fn new() -> Self {
        let host = cpal::default_host();

        Self {
            host,
            state: State::Init,
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

        let handler = match config.sample_format() {
            cpal::SampleFormat::F32 => handlers::Handler::new_f32(device, config),
            cpal::SampleFormat::I16 => unimplemented!(),
            cpal::SampleFormat::U16 => unimplemented!(),
        };

        self.state = State::Ready(handler);
    }

    fn stop(mut self) {
        if let State::Running(stream) = self.state {
            let _ = stream.pause();
        }

        self.state = State::Stopped;
    }

    fn run(mut self) -> Result<Self> {
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
}

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let app = App::new().run()?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(100));
    }
}
