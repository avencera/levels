#[derive(Debug)]
pub struct Amp<T> {
    min: T,
    max: T,
}

impl<T> Amp<T>
where
    T: Copy + Default + PartialOrd + std::fmt::Display,
{
    pub fn new_from_iter(iter: impl Iterator<Item = T>) -> Self {
        let mut min = T::default();
        let mut max = T::default();

        for num in iter {
            if num < min {
                min = num;
            }
            if num > max {
                max = num;
            }
        }

        Self { min, max }
    }
}

pub trait Decibel {
    type Amp;

    fn amp(&self) -> f32;
    fn db(&self) -> f32 {
        self.amp().log10() * 20.0
    }
    fn rounded(&self) -> i32 {
        self.db().round() as i32
    }
}

impl Decibel for Amp<f32> {
    type Amp = Amp<f32>;

    fn amp(&self) -> f32 {
        (self.max - self.min) / 2.0
    }
}

impl Decibel for Amp<i16> {
    type Amp = Amp<i16>;

    fn amp(&self) -> f32 {
        (self.max - self.min) as f32 / 2.0
    }
}

impl Decibel for Amp<u16> {
    type Amp = Amp<u16>;

    fn amp(&self) -> f32 {
        (self.max - self.min) as f32 / 2.0
    }
}
