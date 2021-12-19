mod handlers;

use cpal::traits::{DeviceTrait, HostTrait};
use eyre::Result;

const INTERVAL: u16 = 100;
const LATENCY: u16 = INTERVAL / 2;

struct App {
    handler: handlers::Handler,
}

impl App {
    fn new() -> Self {
        let host = cpal::default_host();
        let device = host
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

        Self { handler }
    }

    fn run(self) -> Result<()> {
        self.handler.run()
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let app = App::new();
    app.run()?;

    Ok(())
}
