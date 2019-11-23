pub use super::fold::*;
pub use super::logger::*;
pub use super::paramcli::*;

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::thread::{spawn, JoinHandle};
use std::time::Duration;
use std::time::SystemTime;

pub struct Explorer {
    ignore_err: bool,
    folder_forbidden_count: u32,
    file_forbidden_count: u32,
    logger: Logger,
}

impl Explorer {
    pub fn new(n : String, o: &Options, to_logger: Sender<String>) -> Explorer {
        let log = Logger::new(n, o.verbose, to_logger);
        Explorer {
            ignore_err: o.ignore_err,
            folder_forbidden_count: 0,
            file_forbidden_count: 0,
            logger: log,
        }
    }

    pub fn run(&mut self, dir: &Path) -> Fold {
        self.logger.verbose(format!("Reading Folder: {:?} ", &dir));
        self.folder_forbidden_count = 0;
        self.file_forbidden_count = 0;
        let start = SystemTime::now();
        let mut d = Fold::new_root(dir);
        self.run_int(dir, &mut d);
        let c = d.get_counts();
        self.logger.timed(
            format!(
                "Folder: {:?} Folder total/forbidden {}/{} Files total/forbidden {}/{}",
                dir, c.0, self.folder_forbidden_count, c.1, self.file_forbidden_count
            ),
            start,
        );
        d
    }

    //internal function called by run so run could do some 1st call things without testing at each runs
    fn run_int(&mut self, dir: &Path, fold: &mut Fold) {
        if dir.is_dir() {
            let dir_content = fs::read_dir(dir);
            match dir_content {
                Err(e) => {
                    self.logger.error(format!("{} on {:?}", e, dir)); //appears on folder of which i have no access right
                    self.folder_forbidden_count += 1;
                    fold.forbidden = true;
                }
                Ok(content) => {
                    for entry in content {
                        match entry {
                            Err(e) => {
                                self.logger.error(format!("{}", e));
                                if !self.ignore_err {
                                    std::process::exit(0);
                                }
                            }
                            Ok(e) => {
                                let path = e.path();
                                if path.is_dir() {
                                    let mut sub_fold = Fold::new(&path);
                                    self.run_int(&path, &mut sub_fold);
                                    fold.add_fold(sub_fold);
                                } else {
                                    let fic = Fic::new(&path);
                                    if fic.is_some() {
                                        fold.add_fic(fic.unwrap());
                                    } else {
                                        self.file_forbidden_count += 1;
                                    }
                                }
                            }
                        };
                    }
                }
            }
        }
    }
}

pub enum Place {
    Src,
    Dst,
}

impl Place {
    pub fn to_string(&self) -> String {
        match self {
            Place::Src => "source".to_string(),
            Place::Dst => "destination".to_string(),
        }
    }
    pub fn clone(&self) -> Place {
        match self {
            Place::Src => Place::Src,
            Place::Dst => Place::Dst,
        }
    }
}

fn start_thread_read(
    what: Place,
    to_join: Sender<(Place, Fold)>,
    data: Vec<PathBuf>,
    opt: &Options,
    to_logger: Sender<String>,
) -> JoinHandle<()> {
    let name = format!("{}_reader", what.to_string());
    let mut explorer = Explorer::new(name.to_string(), opt, to_logger);
    let handle = spawn(move || {
        explorer.logger.starting();
        //timings: elapse count all and the other counts only acting time        
        let start_elapse = SystemTime::now();
        let mut tps = Duration::new(0, 0);
        //number of item receive from chanel
        let mut nb = 0;
        explorer
            .logger
            .log(format!("{} folders to read",  &data.len()));
        //iterate on sources
        for d in data {
            explorer
                .logger
                .verbose(format!("start reading {} ",&d.to_str().unwrap()));
            //read time only
            let start = SystemTime::now();
            let src = explorer.run(&Path::new(d.to_str().unwrap()));
            tps += explorer.logger.timed(
                format!("finished reading {} ",&d.to_str().unwrap()),
                start,
            );
            //send data to join thread thru MPSC chanel
            if to_join.send((what.clone(), src)).is_err() {
                explorer.logger.error("seding data to join".to_string());
                return;
            }
            nb += 1;
        }
        explorer.logger.dual_timed(
            format!("finished all reading {} items", nb),
            tps,
            start_elapse,
        );
    });
    handle
}

/**
 * start a thread that reads sources and send them in a MPSC chanel
 * the sender part of the chanel is the first argument and receive a tuple containing the type (source or destination)
 * and the full contain of one source (folders and files)
 * the second parameter is a list of source to read
*/
pub fn start_thread_read_src(
    to_join: Sender<(Place, Fold)>,
    data: Vec<PathBuf>,
    opt: &Options,
    to_logger: Sender<String>,
) -> JoinHandle<()> {
    start_thread_read(Place::Src, to_join, data, opt, to_logger)
}

/**
 * start a thread that reads destinations and send them in a MPSC chanel
 * the sender part of the chanel is the first argument and receive a tuple containing the type (source or destination)
 * and the full contain of one destination (folders and files)
 * the second parameter is a list of destinations to read
*/
pub fn start_thread_read_dst(
    to_join: Sender<(Place, Fold)>,
    data: Vec<PathBuf>,
    opt: &Options,
    to_logger: Sender<String>,
) -> JoinHandle<()> {
    start_thread_read(Place::Dst, to_join, data, opt, to_logger)
}
