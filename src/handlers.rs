use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Device, InputCallbackInfo, Stream, SupportedStreamConfig,
};
use crossbeam_channel::{Receiver, Sender};
use eyre::Result;
use rtrb::{Consumer, Producer, RingBuffer};

use crate::{
    amp::{Amp, Decibel},
    util, INTERVAL, LATENCY,
};

pub enum Handler {
    F32(InnerHandler<f32>),
    I16(InnerHandler<i16>),
    U16(InnerHandler<u16>),
}

impl Handler {
    pub fn new_f32(device: Device, config: SupportedStreamConfig) -> Self {
        Handler::F32(InnerHandler::new(device, config))
    }

    pub fn new_i16(device: Device, config: SupportedStreamConfig) -> Self {
        Handler::I16(InnerHandler::new(device, config))
    }

    pub fn new_u16(device: Device, config: SupportedStreamConfig) -> Self {
        Handler::U16(InnerHandler::new(device, config))
    }

    pub fn run(self) -> Result<Stream> {
        match self {
            Handler::F32(handler) => Ok(handler.run()?),
            Handler::I16(handler) => Ok(handler.run()?),
            Handler::U16(handler) => Ok(handler.run()?),
        }
    }
}

pub struct InnerHandler<T> {
    device: Device,
    config: SupportedStreamConfig,
    read_at_a_time: usize,
    producer: Producer<T>,
    consumer: Consumer<T>,
}

impl<T> InnerHandler<T>
where
    Amp<T>: Decibel,
    T: Copy + Default + cpal::Sample + PartialOrd + Send + 'static,
{
    fn new(device: Device, config: SupportedStreamConfig) -> Self {
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

    fn run(self) -> Result<Stream> {
        let (sender, receiver): (Sender<Amp<T>>, Receiver<Amp<T>>) =
            crossbeam_channel::bounded(200);

        std::thread::spawn(move || {
            for amp in receiver {
                println!("{}", amp.db());
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
