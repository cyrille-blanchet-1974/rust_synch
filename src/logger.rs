use super::constant::*;
use super::writer::*;

use std::boxed::Box;
use std::fmt;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, SystemTime};

pub struct Logger {
    name: String,
    verbose: bool,
    to_logger: Sender<String>,
}

pub enum MessageType {
    Grave(String),                         //Error so important we stop
    Error(String),                         //Error
    Warning(String),                       //point of attention
    Info(String),                          //normal infos
    Verbose(String), //verbose informaton (only shown with /verbose switch in CLI)
    Timed(String, Duration), //versbose with a duration
    DualTimed(String, Duration, Duration), //verbose with two duration: real and elapse
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageType::Grave(s) => write!(f, "!Major Error! ==> {}. STOPPING", s),
            MessageType::Error(s) => write!(f, "!Error! ==> {}", s),
            MessageType::Warning(s) => write!(f, "Warning ==> {}", s),
            MessageType::Info(s) => write!(f, "{}", s),
            MessageType::Verbose(s) => write!(f, "{}", s),
            MessageType::Timed(s, t) => write!(f, "{} in {:?}", s, t),
            MessageType::DualTimed(s, t, te) => write!(f, "{} in {:?}/{:?} ", s, t, te),
        }
    }
}

impl Logger {
    pub fn new(name: String, verbose: bool, to_logger: Sender<String>) -> Logger {
        Logger {
            name,
            verbose,
            to_logger,
        }
    }

    fn send(&self, data: MessageType) {
        if self
            .to_logger
            .send(format!("{} says: '{}'", self.name, data))
            .is_err()
        {
            println!("Erreur sending log");
        }
    }

    pub fn terminating(&self, data: String, error: i32) {
        self.send(MessageType::Grave(data.clone()));
        //also display on console
        println!("{}", MessageType::Grave(data));
        std::process::exit(error);
    }

    pub fn starting(&self) {
        self.send(MessageType::Info("Starting".to_string()));
    }

    pub fn log(&self, data: String) {
        self.send(MessageType::Info(data));
    }

    pub fn error(&self, data: String) {
        self.send(MessageType::Error(data));
    }

    pub fn warning(&self, data: String) {
        self.send(MessageType::Warning(data));
    }

    pub fn verbose(&self, data: String) {
        if self.verbose {
            self.send(MessageType::Verbose(data));
        }
    }

    pub fn timed(&self, data: String, start: SystemTime) -> Duration {
        if self.verbose {
            let end = SystemTime::now();
            let tps = end
                .duration_since(start)
                .expect("ERROR computing duration!");
            self.send(MessageType::Timed(data, tps));
            return tps;
        }
        Duration::new(0, 0)
    }

    pub fn dual_timed(&self, data: String, real: Duration, start_elapse: SystemTime) -> Duration {
        let end = SystemTime::now();
        let tps = end
            .duration_since(start_elapse)
            .expect("ERROR computing duration!");
        self.send(MessageType::DualTimed(data, real, tps));
        tps
    }
}

/**
* start logger thread
* for now we only output what we receive to console
*
*/
pub fn start_thread_logger(from_all: Receiver<String>, output: PathBuf) -> JoinHandle<()> {
    spawn(move || {
        let mut writer: Box<dyn Writing>;
        match WriterDisk::new(output, true) {
            Some(w) => {
                writer = Box::new(w);
            }
            None => {
                //no logger ?
                writer = Box::new(WriterConsole::new());
            }
        };
        let start_elapse = SystemTime::now();
        let mut tps = Duration::new(0, 0);
        for data in from_all {
            let start = SystemTime::now();
            writer.as_mut().write(data);
            let end = SystemTime::now();
            tps += end
                .duration_since(start)
                .expect("ERROR computing duration!");
        }
        let end_elapse = SystemTime::now();
        let tps_elapse = end_elapse
            .duration_since(start_elapse)
            .expect("ERROR computing duration!");
        let nb_ecr = writer.as_mut().get_nb_ecr();
        writer.as_mut().write(
            format!(
                "{} says: '{} lines writes in {:?}/{:?}'",
                LOGGER.to_string(),
                nb_ecr,
                tps,
                tps_elapse
            )
            .to_string(),
        );
    })
}
