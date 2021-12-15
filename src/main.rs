use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat,
};
use ringbuf::{Consumer, Producer, RingBuffer};

const INTERVAL: u16 = 100;
const LATENCY: u16 = INTERVAL / 2;

struct App {
    device: Device,
    channels: u16,
    sample_rate: u32,
    read_at_a_time: usize,
    producer: Producer<f32>,
    consumer: Consumer<f32>,
    sample_format: SampleFormat,
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

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        let read_at_a_time =
            ((sample_rate as f32 / 1000.0) * (channels * INTERVAL) as f32).ceil() as usize;

        let write_ahead =
            ((sample_rate as f32 / 1000.0) * (channels * LATENCY) as f32).ceil() as usize;

        let ring = RingBuffer::new(read_at_a_time + write_ahead);
        let (producer, consumer) = ring.split();

        Self {
            channels,
            sample_rate,
            read_at_a_time,
            producer,
            consumer,
            sample_format: config.sample_format(),
        }
    }

    fn handle_data(&mut self, data: &[f32]) {
        if self.producer.push_slice(data) < data.len() {
            eprintln!("buffer too slow")
        }
    }
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => {
            device.build_input_stream(&config.into(), input_data_fn, err_fn)?
        }
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i16>(data),
            err_fn,
        )?,
        cpal::SampleFormat::U16 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<u16>(data),
            err_fn,
        )?,
    };

    stream.play()?;

    let mut input = [0.0_f32; READ_BUFFER];
    let thread = std::thread::spawn(move || loop {
        use std::cmp::Ordering::Equal;

        consumer.pop_slice(&mut input);

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

        std::thread::sleep(std::time::Duration::from_millis(MS as u64));
    });

    thread.join().expect("shouldn't finish first");

    Ok(())
}

fn write_input_data<T>(input: &[T])
where
    T: cpal::Sample + core::fmt::Debug + std::fmt::Display + std::cmp::Ord,
{
    let max = input.iter().max().unwrap();
    let min = input.iter().min().unwrap();

    println!("MIN: {}, MAX: {}", min, max)
}
