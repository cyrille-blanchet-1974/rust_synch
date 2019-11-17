use std::sync::mpsc::{Receiver,Sender};
use std::thread::{spawn, JoinHandle};
use std::time::SystemTime;

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

    pub fn log(&self, data: String) 
    {
        if self.to_logger.send(data).is_err() {
            println!("Erreur sending log");
        }
    }
    
    pub fn log_verbose(&self, data: String) 
    {
        if self.verbose 
        {
            self.log(data);
        }
    }

    pub fn log_timed_verbose(&self, data: String, start: SystemTime) 
    {
        if self.verbose 
        {
            let end = SystemTime::now();
            let tps = end
                .duration_since(start)
                .expect("ERROR computing duration!");
            self.log(format!("{} Duration {:?}",data,tps));
        }
    }
}


/**
* start logger thread
* for now we only output what we receive to console
* 
*/
pub fn start_thread_logger(from_all: Receiver<String>) -> JoinHandle<()>
{
   let handle = spawn( move || {
       println!("INFO start logger");
       for data in from_all{
            println!("{}", data);
       }
   });
   println!("INFO logger ends");
   handle
}
