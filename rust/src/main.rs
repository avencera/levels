use levels::{DecibelResponder, Levels};

struct Responder {}

impl DecibelResponder for Responder {
    fn decibel(&self, decibel: i32) {
        println!("{}", decibel);
    }
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let levels = Levels::new();
    levels.run(Box::new(Responder {}));

    loop {
        std::thread::sleep(std::time::Duration::from_secs(100));
    }
}
