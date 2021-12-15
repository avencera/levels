use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::RingBuffer;

const SAMPLE: i32 = 48;
const MS: i32 = 100;
const CHANNELS: i32 = 2;

const READ_BUFFER: usize = (((SAMPLE + 1) * CHANNELS) * MS) as usize;
const WRITE_BUFFER: usize = ((SAMPLE + 1) * MS * CHANNELS) as usize;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("no input device available");

    // let mut supported_configs_range = device
    //     .supported_input_configs()
    //     .expect("error while querying configs");

    let config = device
        .default_input_config()
        .expect("Failed to get default input config");

    println!("Default input config: {:?}", config);

    println!("CHANNELS: {}", config.channels());
    println!("SAMPLE RATE: {}", config.sample_rate().0);
    println!("SAMPLE FORMAT: {:#?}", config.sample_format());

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    let ring = RingBuffer::new(WRITE_BUFFER);
    let (mut producer, mut consumer) = ring.split();

    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        if producer.push_slice(data) < data.len() {
            eprintln!("buffer too slow")
        }
    };

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

    let thread = std::thread::spawn(move || loop {
        use std::cmp::Ordering::Equal;

        let mut input = [0.0_f32; READ_BUFFER];

        consumer.pop_slice(input.as_mut_slice());

        println!("BUFFER LEFT: {}", consumer.remaining());

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
