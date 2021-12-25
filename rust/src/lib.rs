mod amp;
mod app;
pub mod handler;
mod util;

use std::sync::Mutex;

pub type App = app::App;

const INTERVAL: u16 = 100;
const LATENCY: u16 = 50;

pub trait OnCallAnswered {
    fn hello(&self) -> String;
    fn busy(&self);
    fn text_received(&self, text: String);
}

#[derive(Debug, Clone)]
struct Telephone;
impl Telephone {
    fn new() -> Self {
        Telephone
    }
    fn call(&self, domestic: bool, call_responder: Box<dyn OnCallAnswered>) {
        if domestic {
            let _ = call_responder.hello();
        } else {
            call_responder.busy();
            call_responder.text_received("Not now, I'm on another call!".into());
        }
    }
}

pub trait DecibelResponder {
    fn decibel(&self, decibel: i32);
}

pub struct AppInterface {
    app: Mutex<App>,
}

impl AppInterface {
    fn new() -> Self {
        Self {
            app: Mutex::new(App::new()),
        }
    }

    // fn stop(&self) {
    //     let mut app = self.app.write().unwrap();
    //     *app.stop();
    // }

    fn run(&self) {
        let mut app = self.app.lock().unwrap();
        (*app).run().unwrap();
    }
}

impl Default for AppInterface {
    fn default() -> Self {
        Self::new()
    }
}

include!(concat!(env!("OUT_DIR"), "/decibel.uniffi.rs"));
