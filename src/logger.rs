use super::writer::*;

use std::boxed::Box;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, SystemTime};

pub struct Logger {
    verbose: bool,
    to_logger: Sender<String>,
}

impl Logger {
    pub fn new(verbose: bool, to_logger: Sender<String>) -> Logger {
        Logger {
            verbose: verbose,
            to_logger: to_logger,
        }
    }

    pub fn log(&self, data: String) {
        if self.to_logger.send(data).is_err() {
            println!("Erreur sending log");
        }
    }

    pub fn log_verbose(&self, data: String) {
        if self.verbose {
            self.log(data);
        }
    }

    pub fn log_timed_verbose(&self, data: String, start: SystemTime) {
        if self.verbose {
            let end = SystemTime::now();
            let tps = end
                .duration_since(start)
                .expect("ERROR computing duration!");
            self.log(format!("{} Duration {:?}", data, tps));
        }
    }
}

/**
* start logger thread
* for now we only output what we receive to console
*
*/
pub fn start_thread_logger(from_all: Receiver<String>, output: PathBuf) -> JoinHandle<()> {
    let handle = spawn(move || {
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
                "INFO {} lignes writes in {:?}/{:?}",
                nb_ecr, tps, tps_elapse
            )
            .to_string(),
        );
    });
    handle
}
