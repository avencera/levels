use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Device, InputCallbackInfo, SupportedStreamConfig,
};
use eyre::Result;
use ringbuf::{Consumer, Producer, RingBuffer};

use crate::{INTERVAL, LATENCY};

pub enum Handler {
    F32(F32Handler),
}

impl Handler {
    pub fn new_f32(device: Device, config: SupportedStreamConfig) -> Self {
        Handler::F32(F32Handler::new(device, config))
    }

    pub fn run(self) -> Result<()> {
        match self {
            Handler::F32(handler) => handler.run(),
        }
    }
}

pub struct F32Handler {
    device: Device,
    config: SupportedStreamConfig,
    read_at_a_time: usize,
    producer: Producer<f32>,
    consumer: Consumer<f32>,
}

impl F32Handler {
    fn new(device: Device, config: SupportedStreamConfig) -> Self {
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        let read_at_a_time =
            ((sample_rate as f32 / 1000.0) * (channels * INTERVAL) as f32).ceil() as usize;

        let write_ahead =
            ((sample_rate as f32 / 1000.0) * (channels * LATENCY) as f32).ceil() as usize;

        let ring = RingBuffer::new(read_at_a_time + write_ahead);
        let (producer, consumer) = ring.split();

        Self {
            device,
            config,
            read_at_a_time,
            producer,
            consumer,
        }
    }

    fn run(self) -> Result<()> {
        let mut input = Vec::with_capacity(self.read_at_a_time);

        let mut consumer = self.consumer;
        let mut producer = self.producer;

        let stream = self.device.build_input_stream(
            &self.config.into(),
            move |data, _: &InputCallbackInfo| {
                if producer.push_slice(data) < data.len() {
                    eprintln!("buffer too slow")
                }
            },
            move |err| {
                eprintln!("an error occurred on stream: {}", err);
            },
        )?;

        stream.play()?;

        std::thread::spawn(move || loop {
            use std::cmp::Ordering::Equal;

            (consumer).pop_slice(&mut input);

            let avg: f32 = input.iter().sum::<f32>() / input.len() as f32;

            let max = input
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(Equal))
                .unwrap();

            let min = input
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(Equal))
                .unwrap();

            println!("AMP MIN: {}, AMP MAX: {}", avg - min, max - avg);

            std::thread::sleep(std::time::Duration::from_millis(INTERVAL as u64));
        });

        Ok(())
    }
}
