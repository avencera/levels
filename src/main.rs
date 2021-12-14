use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

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

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<f32>(data),
            err_fn,
        )?,
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

    loop {
        std::thread::sleep(std::time::Duration::from_secs(200));
    }
}

fn write_input_data<T>(input: &[T])
where
    T: cpal::Sample + core::fmt::Debug,
{
    println!("T: {:#?}", input)
}
