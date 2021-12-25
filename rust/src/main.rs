use levels::App;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let _app = App::new().run()?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(100));
    }
}
