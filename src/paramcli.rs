pub use super::readconf::*;

use std::env;

#[derive(Debug)]
pub struct Options
//options from command line (subset of Paramcli)
{
    pub verbose: bool,
    pub crypt: bool,
    pub ignore_err: bool,
}

#[derive(Debug)]
pub struct Paramcli
//parameters from command line and/or confFile
{
    pub source: Vec<String>,
    pub destination: Vec<String>,
    pub fic_out: String,
    pub verbose: bool,
    pub crypt: bool,
    pub ignore_err: bool,
    pub config: String,
}

impl Paramcli {
    pub fn new() -> Paramcli {
        let mut fic = String::new();
        let mut ver = false;
        let mut cry = false;
        let mut ign = false;
        let mut src = Vec::new();
        let mut dst = Vec::new();
        let mut conf = String::new();

        let args: Vec<String> = env::args().skip(1).collect();
        if args.len() == 0 {
            help();
        }
        for arg in args {
            println!("{}", arg);
            if arg == "/?"
                || arg == "-?"
                || arg.to_lowercase() == "/help"
                || arg.to_lowercase() == "-help"
            {
                help();
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
        }
        if !conf.is_empty() {
            let readconf = Readconf::new(&conf);
            src = readconf.source;
            dst = readconf.destination;
        }
        //checks
        if src.len() != dst.len() {
            println!("ERROR! number of source and destination do not match!");
            println!("--------------------------------------------------");
            help();
        }
        if src.len() == 0 {
            println!("ERROR! nothing to synch!");
            println!("--------------------------------------------------");
            help();
        }
        for i in 1..src.len() {
            if src.get(i) == dst.get(i) {
                println!("ERROR! src equals to destination {:?}", src.get(i));
                println!("--------------------------------------------------");
                help();
            }
        }
        if fic.is_empty() {
            println!("ERROR! not output script provided");
            println!("--------------------------------------------------");
            help();
        }
        Paramcli {
            source: src,
            destination: dst,
            fic_out: fic,
            verbose: ver,
            crypt: cry,
            ignore_err: ign,
            config: conf,
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
        }
    }
}

fn get_param(arg: String) -> String {
    let mut res = String::new();
    for part in arg.split(":").skip(1) {
        if !res.is_empty() {
            res.push_str(":");
        }
        res.push_str(part);
    }
    if arg.ends_with(":") {
        res.push_str(":");
    }
    res
}

fn help() {
    println!("syntax 1 : synch /src:folder_src /dst:folder_dst /fic:output_script [/verbose] [/crypt] [/ignore_err]");
    println!("syntax 2 : synch /conf:conf_file                 /fic:output_script [/verbose] [/crypt] [/ignore_err]");
    println!("paramerters between [] are optionnals");
    println!("------------------------------------");
    println!("folder_src: folder to be duplicate");
    println!("folder_dst: destination folder (will become a perfect clone of folder_src)");
    println!("conf_file: file containing multiple sources and destinations folders *");
    println!("output_script: script filte where we write commands to clone src to dst");
    println!("/verbose: display more information on the screen");
    println!("/crypt: if a source file is exactly 4096 bytes less than the destination one then concider them equals");
    println!("/ignore_err: do not stop in case of error");
    println!("---------------------------------------------------------------------------");
    println!("* conf_file format: multiple lines working in pairs");
    println!("lines starting with 'source='  or 'destination='");
    std::process::exit(0);
}
