use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Device, InputCallbackInfo, Stream, SupportedStreamConfig,
};
use crossbeam_channel::{Receiver, Sender};
use eyre::Result;
use rtrb::{Consumer, CopyToUninit, Producer, RingBuffer};

use crate::{
    amp::{Amp, Decibel},
    INTERVAL, LATENCY,
};

pub enum Handler {
    F32(F32Handler),
    I16(I16Handler),
}

impl Handler {
    pub fn new_f32(device: Device, config: SupportedStreamConfig) -> Self {
        Handler::F32(F32Handler::new(device, config))
    }

    pub fn new_i16(device: Device, config: SupportedStreamConfig) -> Self {
        Handler::I16(I16Handler::new(device, config))
    }

    pub fn run(self) -> Result<Stream> {
        let (sender, receiver): (Sender<f32>, Receiver<f32>) = crossbeam_channel::bounded(200);

        std::thread::spawn(move || {
            for decibel in receiver {
                println!("{}", decibel);
            }
        });

        match self {
            Handler::F32(handler) => Ok(handler.run(sender)?),
            Handler::I16(handler) => Ok(handler.run(sender)?),
        }
    }
}

pub struct F32Handler {
    inner: InnerHandler,
    producer: Producer<f32>,
    consumer: Consumer<f32>,
}

impl F32Handler {
    fn new(device: Device, config: SupportedStreamConfig) -> Self {
        let inner = InnerHandler::new(device, config);
        let (producer, consumer) = RingBuffer::new(inner.buffer_size);

        Self {
            inner,
            producer,
            consumer,
        }
    }

    fn run(self, sender: Sender<f32>) -> Result<Stream> {
        self.inner.run(self.consumer, self.producer, sender)
    }
}

pub struct I16Handler {
    inner: InnerHandler,
    producer: Producer<i16>,
    consumer: Consumer<i16>,
}

impl I16Handler {
    fn new(device: Device, config: SupportedStreamConfig) -> Self {
        let inner = InnerHandler::new(device, config);
        let (producer, consumer) = RingBuffer::new(inner.buffer_size);

        Self {
            inner,
            producer,
            consumer,
        }
    }

    fn run(self, sender: Sender<f32>) -> Result<Stream> {
        self.inner.run(self.consumer, self.producer, sender)
    }
}

pub struct InnerHandler {
    device: Device,
    config: SupportedStreamConfig,
    read_at_a_time: usize,
    buffer_size: usize,
}

impl InnerHandler {
    fn new(device: Device, config: SupportedStreamConfig) -> Self {
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        let read_at_a_time =
            ((sample_rate as f32 / 1000.0) * (channels * INTERVAL) as f32).ceil() as usize;

        let write_ahead =
            ((sample_rate as f32 / 1000.0) * (channels * LATENCY) as f32).ceil() as usize;

        let buffer_size = read_at_a_time + write_ahead;

        Self {
            device,
            config,
            read_at_a_time,
            buffer_size,
        }
    }

    fn run<T>(
        self,
        mut consumer: Consumer<T>,
        mut producer: Producer<T>,
        sender: Sender<f32>,
    ) -> Result<Stream>
    where
        Amp<T>: Decibel,
        T: Copy
            + Default
            + cpal::Sample
            + PartialEq
            + PartialOrd
            + Send
            + 'static
            + std::ops::Add<Output = T>
            + std::ops::Div<Output = T>,
    {
        let stream = self.device.build_input_stream(
            &self.config.into(),
            move |data: &[_], _: &InputCallbackInfo| {
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

            sender.send(amp.db()).expect("sender will always be ready");

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
