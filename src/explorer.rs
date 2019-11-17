pub use super::fold::*;
pub use super::paramcli::*;
pub use super::logger::*;

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
    pub fn new(o: &Options,to_logger: Sender<String>) -> Explorer {
        let log = Logger::new(o.verbose,to_logger);
        Explorer {
            ignore_err: o.ignore_err,
            folder_forbidden_count: 0,
            file_forbidden_count: 0,
            logger: log,
        }
    }

    pub fn run(&mut self, dir: &Path) -> Fold {
        self.logger.log_verbose(format!("TRACE Reading Folder: {:?} ", &dir));
        self.folder_forbidden_count = 0;
        self.file_forbidden_count = 0;
        let start = SystemTime::now();
        let mut d = Fold::new_root(dir);
        self.run_int(dir, &mut d);
        let c = d.get_counts();
        self.logger.log_timed_verbose(format!("TRACE Folder: {:?} Folder total/forbidden {}/{} Files total/forbidden {}/{}",dir,c.0,self.folder_forbidden_count,c.1,self.file_forbidden_count),start);
        d
    }

    //internal function called by run so run could do some 1st call things without testing at each runs
    fn run_int(&mut self, dir: &Path, fold: &mut Fold) {
        if dir.is_dir() {
            let dir_content = fs::read_dir(dir);
            match dir_content {
                Err(e) => {
                    self.logger.log(format!("ERROR {} on {:?}", e, dir)); //appears on folder of which i have no access right
                    self.folder_forbidden_count += 1;
                    fold.forbidden = true;
                }
                Ok(content) => {
                    for entry in content {
                        match entry {
                            Err(e) => {
                                self.logger.log(format!("ERROR {}", e));
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
    to_logger: Sender<String>
) -> JoinHandle<()> {
    let mut explorer = Explorer::new(opt, to_logger);
    let name = format!("{}_reader",what.to_string());
    explorer.logger.log(format!("INFO start {}",&name));
    let handle = spawn(move || {
        //timings: elapse count all and the other counts only acting time
        let start_elapse = SystemTime::now();
        let mut tps = Duration::new(0, 0);
        //number of item receive from chanel
        let mut nb = 0;
        explorer.logger.log(format!("INFO {} start reading {} folders in {}s",&name,&data.len(),what.to_string()));        
        //iterate on sources
        for d in data {
            //read time only
            let start = SystemTime::now();
            let src = explorer.run(&Path::new(d.to_str().unwrap()));
            let end = SystemTime::now();
            tps += end
                .duration_since(start)
                .expect("ERROR computing duration!");
            //send data to join thread thru MPSC chanel
            if to_join.send((what.clone(), src)).is_err() {
                println!("ERROR in start_read_{}", what.to_string());
                return;
            }
            nb += 1;
        }
        let end_elapse = SystemTime::now();
        let tps_elapse = end_elapse
            .duration_since(start_elapse)
            .expect("ERROR computing duration!");
        explorer.logger.log(format!("INFO {} {} folders read in {:?}/{:?}",&name,nb,tps,tps_elapse));
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
    to_logger: Sender<String>
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
    to_logger: Sender<String>
) -> JoinHandle<()> {
    start_thread_read(Place::Dst, to_join, data, opt, to_logger)
}
