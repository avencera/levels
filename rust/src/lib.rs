mod amp;
mod app;
pub mod handler;
mod util;

use crossbeam_channel::Sender;

pub type App = app::App;

const INTERVAL: u16 = 100;
const LATENCY: u16 = 50;

pub trait DecibelResponder: Send {
    fn decibel(&self, decibel: i32, color: Color);
}

pub struct Levels {
    actor: Sender<Msg>,
}

pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    SkyBlue,
}

enum Msg {
    Start(Box<dyn DecibelResponder>),
    Stop,
}

impl Levels {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::bounded(200);

        std::thread::spawn(move || {
            let mut app = app::App::new();

            for msg in receiver {
                match msg {
                    Msg::Start(responder) => app.run(responder).unwrap(),
                    Msg::Stop => {
                        app.stop();
                    }
                }
            }
        });

        Self { actor: sender }
    }

    pub fn stop(&self) {
        self.actor.send(Msg::Stop).unwrap();
    }

    pub fn run(&self, responder: Box<dyn DecibelResponder>) {
        self.actor.send(Msg::Start(responder)).unwrap();
    }
}

impl Default for Levels {
    fn default() -> Self {
        Self::new()
    }
}

include!(concat!(env!("OUT_DIR"), "/decibel.uniffi.rs"));
