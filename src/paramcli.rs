pub use super::readconf::*;

use std::env;

#[derive(Debug)]
pub struct Options
//options from command line (subset of Paramcli)
{
    pub verbose: bool,
    pub crypt: bool,
    pub ignore_err: bool,
    pub exceptions: Vec<String>,
    pub ignore_date_diff: bool,
}

#[derive(Debug)]
pub struct Paramcli
//parameters from command line and/or confFile
{
    pub source: Vec<String>,
    pub destination: Vec<String>,
    pub exceptions: Vec<String>,
    pub fic_out: String,
    pub verbose: bool,
    pub crypt: bool,
    pub ignore_err: bool,
    pub config: String,
    pub ignore_date_diff: bool,
}

impl Paramcli {
    pub fn new() -> Paramcli {
        let mut fic = String::new();
        let mut ver = false;
        let mut cry = false;
        let mut ign = false;
        let mut src = Vec::new();
        let mut dst = Vec::new();
        let mut exc = Vec::new();
        let mut conf = String::new();
        let mut igndate = false;

        let args: Vec<String> = env::args().skip(1).collect();
        let name = env::args()
            .take(1)
            .next()
            .unwrap_or_else(|| String::from("synch"));
        println!("{} 1.1.0 (2021)", name);
        if args.is_empty() {
            help(&name);
        }
        for arg in args {
            println!("{}", arg);
            if arg == "/?"
                || arg == "-?"
                || arg.to_lowercase() == "/help"
                || arg.to_lowercase() == "-help"
            {
                help(&name);
            }
            if arg.to_lowercase().starts_with("/src:") {
                src.push(get_param(arg));
                continue;
            }
            if arg.to_lowercase().starts_with("/dst:") {
                dst.push(get_param(arg));
                continue;
            }
            if arg.to_lowercase().starts_with("/conf:") {
                conf = get_param(arg);
                continue;
            }
            if arg.to_lowercase().starts_with("/fic:") {
                fic = get_param(arg);
                continue;
            }
            if arg.to_lowercase() == "/verbose" {
                ver = true;
                continue;
            }
            if arg.to_lowercase() == "/crypt" {
                cry = true;
                continue;
            }
            if arg.to_lowercase() == "/ignore_err" {
                ign = true;
                continue;
            }
            if arg.to_lowercase() == "/ignore_date_diff" {
                igndate = true;
                continue;
            }
        }
        if !conf.is_empty() {
            let readconf = Readconf::new(&conf);
            src = readconf.source;
            dst = readconf.destination;
            exc = readconf.exception;
        }
        //checks
        if src.len() != dst.len() {
            println!("ERROR! number of source and destination do not match!");
            println!("--------------------------------------------------");
            help(&name);
        }
        if src.is_empty() {
            println!("ERROR! nothing to synch!");
            println!("--------------------------------------------------");
            help(&name);
        }
        for i in 1..src.len() {
            if src.get(i) == dst.get(i) {
                println!("ERROR! src equals to destination {:?}", src.get(i));
                println!("--------------------------------------------------");
                help(&name);
            }
        }
        if fic.is_empty() {
            println!("ERROR! not output script provided");
            println!("--------------------------------------------------");
            help(&name);
        }
        Paramcli {
            source: src,
            destination: dst,
            exceptions: exc,
            fic_out: fic,
            verbose: ver,
            crypt: cry,
            ignore_err: ign,
            config: conf,
            ignore_date_diff: igndate,
        }
    }
    /**
     * return a new Paramcli clone of self
     * but without source, destination, fic_out and config
     */
    pub fn to_options(&self) -> Options {
        Options {
            verbose: self.verbose,
            crypt: self.crypt,
            ignore_err: self.ignore_err,
            exceptions: self.exceptions.clone(),
            ignore_date_diff: self.ignore_date_diff,
        }
    }
}

fn get_param(arg: String) -> String {
    let mut res = String::new();
    for part in arg.split(':').skip(1) {
        if !res.is_empty() {
            res.push(':');
        }
        res.push_str(part);
    }
    if arg.ends_with(':') {
        res.push(':');
    }
    res
}

fn help(name: &str) {
    println!("syntax 1 : {} /src:folder_src /dst:folder_dst /fic:output_script [/verbose] [/crypt] [/ignore_err] [/ignore_date_diff]",name);
    println!("syntax 2 : {} /conf:conf_file                 /fic:output_script [/verbose] [/crypt] [/ignore_err] [/ignore_date_diff]",name);
    println!("paramerters between [] are optionnals");
    println!("------------------------------------");
    println!("folder_src: folder to be duplicate");
    println!("folder_dst: destination folder (will become a perfect clone of folder_src)");
    println!("conf_file: file containing multiple sources and destinations folders *");
    println!("output_script: script filte where we write commands to clone src to dst");
    println!("/verbose: display more information on the screen");
    println!("/crypt: if a source file is exactly 4096 bytes less than the destination one then concider them equals");
    println!("/ignore_err: do not stop in case of error");
    println!("/ignore_date_diff: do not copy file with only date diff");
    println!("---------------------------------------------------------------------------");
    println!("* conf_file format: multiple lines working in pairs");
    println!("lines starting with 'source='  or 'destination='");
    println!(
        "lines can also start with 'exception=' folder containing these values will be ignore"
    );
    std::process::exit(0);
}
