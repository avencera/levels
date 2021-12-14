use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::RingBuffer;

const SAMPLE: i32 = 44;
const MS: i32 = 50;

const SIZE: usize = ((SAMPLE + 6) * MS) as usize;
const BUFFER: usize = SIZE * 2 as usize;

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

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    let ring = RingBuffer::new(BUFFER + 1024);
    let (mut producer, mut consumer) = ring.split();

    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        if producer.push_slice(data) < 512 {
            eprintln!("output stream fell behind: try increasing latency");
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

        let mut input = [0.0_f32; SIZE];

        consumer.pop_slice(input.as_mut_slice());

        let max = input
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Equal))
            .unwrap();

        let min = input
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(Equal))
            .unwrap();

        println!("MIN: {}, MAX: {}", min, max);

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

fn get_max(mut producer: ringbuf::Producer<f32>, input: &[f32]) {
    for &sample in input {
        producer.push(sample).unwrap()
    }

    // use std::cmp::Ordering::Equal;
    // let max = input
    //     .iter()
    //     .max_by(|a, b| a.partial_cmp(b).unwrap_or(Equal))
    //     .unwrap();

    // let min = input
    //     .iter()
    //     .min_by(|a, b| a.partial_cmp(b).unwrap_or(Equal))
    //     .unwrap();

    // println!("MIN: {}, MAX: {}", min, max)
}
