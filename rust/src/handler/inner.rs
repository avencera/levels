use std::fmt::{Debug, Display};

use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Device, InputCallbackInfo, Stream, SupportedStreamConfig,
};
use crossbeam_channel::{Receiver, Sender};
use eyre::Result;
use rtrb::{Consumer, Producer, RingBuffer};

use crate::{
    amp::{Amp, Decibel},
    util, Color, DecibelResponder, INTERVAL, LATENCY,
};

pub struct InnerHandler<T> {
    device: Device,
    config: SupportedStreamConfig,
    read_at_a_time: usize,
    producer: Producer<T>,
    consumer: Consumer<T>,
}

impl<T> InnerHandler<T>
where
    Amp<T>: Decibel + Send + Sync + 'static,
    T: Copy + Default + cpal::Sample + PartialOrd + Send + 'static + Sync + Display + Debug,
{
    pub fn new(device: Device, config: SupportedStreamConfig) -> Self {
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        let read_at_a_time =
            ((sample_rate as f32 / 1000.0) * (channels * INTERVAL) as f32).ceil() as usize;

        let write_ahead =
            ((sample_rate as f32 / 1000.0) * (channels * LATENCY) as f32).ceil() as usize;

        let buffer_size = read_at_a_time + write_ahead;

        let (producer, consumer) = RingBuffer::new(buffer_size);

        Self {
            device,
            config,
            read_at_a_time,
            producer,
            consumer,
        }
    }

    pub fn run(self, responder: Box<dyn DecibelResponder>) -> Result<Stream> {
        let (sender, receiver): (Sender<Amp<T>>, Receiver<Amp<T>>) =
            crossbeam_channel::bounded(1000);

        std::thread::spawn(move || {
            for amp in receiver {
                let amp = amp.rounded();
                let color = match amp {
                    i32::MIN..=-21 => Color::Blue,
                    -20..=-13 => Color::SkyBlue,
                    -12..=-8 => Color::Green,
                    -7..=-2 => Color::Yellow,
                    -1..=i32::MAX => Color::Red,
                };

                responder.decibel(amp, color);
            }
        });

        self.start_stream(sender)
    }

    fn start_stream(self, sender: Sender<Amp<T>>) -> Result<Stream> {
        let mut consumer = self.consumer;
        let mut producer = self.producer;

        let stream = self.device.build_input_stream(
            &self.config.into(),
            move |data: &[_], _: &InputCallbackInfo| {
                let items_left = util::push_partial_slice(&mut producer, data);
                if items_left != 0 {
                    log::warn!("buffer hasn't been cleared, items left {}", items_left);
                }
            },
            move |err| {
                log::error!("an error occurred on stream: {}", err);
            },
        )?;

        stream.play()?;

        std::thread::spawn(move || loop {
            if consumer.is_abandoned() {
                break;
            }

            let input = match consumer.read_chunk(self.read_at_a_time) {
                Ok(read_chunk) => read_chunk.into_iter(),
                Err(rtrb::chunks::ChunkError::TooFewSlots(available)) => consumer
                    .read_chunk(available)
                    .expect("will always have available")
                    .into_iter(),
            };

            let amp = Amp::new_from_iter(input);

            sender.send(amp).expect("sender will always be ready");

            std::thread::sleep(std::time::Duration::from_millis(INTERVAL as u64));
        });

        Ok(stream)
    }
}
