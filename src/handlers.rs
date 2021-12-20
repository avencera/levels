use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Device, InputCallbackInfo, Stream, SupportedStreamConfig,
};
use crossbeam_channel::{Receiver, Sender};
use eyre::Result;
use rtrb::{Consumer, CopyToUninit, Producer, RingBuffer};

use crate::{INTERVAL, LATENCY};

pub enum Handler {
    F32(F32Handler),
}

impl Handler {
    pub fn new_f32(device: Device, config: SupportedStreamConfig) -> Self {
        Handler::F32(F32Handler::new(device, config))
    }

    pub fn run(self) -> Result<Stream> {
        let (sender, receiver): (Sender<Amp<f32>>, Receiver<Amp<f32>>) =
            crossbeam_channel::bounded(100);

        std::thread::spawn(move || {
            for amp in receiver {
                println!("AVG {}", amp.avg());
            }
        });

        match self {
            Handler::F32(handler) => Ok(handler.run(sender)?),
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

        let buffer_size = read_at_a_time + write_ahead;
        println!("BUFFER SIZE: {}", buffer_size);

        let (producer, consumer) = RingBuffer::new(buffer_size);

        Self {
            device,
            config,
            read_at_a_time,
            producer,
            consumer,
        }
    }

    fn run(self, sender: Sender<Amp<f32>>) -> Result<Stream> {
        let mut consumer = self.consumer;
        let mut producer = self.producer;

        let stream = self.device.build_input_stream(
            &self.config.into(),
            move |data: &[f32], _: &InputCallbackInfo| {
                let items_left = push_partial_slice(&mut producer, data);
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

fn push_partial_slice<T>(queue: &mut Producer<T>, slice: &[T]) -> usize
where
    T: Copy,
{
    use rtrb::chunks::ChunkError::TooFewSlots;

    let slice_len = slice.len();

    let mut chunk = match queue.write_chunk_uninit(slice_len) {
        Ok(chunk) => chunk,
        // Remaining slots are returned, this will always succeed:
        Err(TooFewSlots(n)) => queue.write_chunk_uninit(n).unwrap(),
    };

    let end = chunk.len();
    let (first, second) = chunk.as_mut_slices();
    let mid = first.len();
    slice[..mid].copy_to_uninit(first);
    slice[mid..end].copy_to_uninit(second);

    // SAFETY: All slots have been initialized
    unsafe {
        chunk.commit_all();
    }

    slice_len - end
}

#[derive(Debug)]
struct Amp<T> {
    min: T,
    max: T,
    sum: T,
    len: usize,
}

impl<T> Amp<T>
where
    T: Copy
        + Default
        + PartialEq
        + PartialOrd
        + core::fmt::Debug
        + std::ops::Add<Output = T>
        + std::ops::Div<Output = T>,
{
    fn new_from_iter(iter: impl Iterator<Item = T>) -> Self {
        let mut min = T::default();
        let mut max = T::default();
        let mut sum = T::default();
        let mut len: usize = 0;

        for num in iter {
            if num < min {
                min = num;
            }
            if num > max {
                max = num;
            }

            sum = sum + num;
            len += 1
        }

        Self { min, max, sum, len }
    }
}

impl Amp<f32> {
    pub fn avg(self) -> f32 {
        self.sum / self.len as f32
    }
}
