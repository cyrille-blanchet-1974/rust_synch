use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Debug)]
pub struct Readconf {
    pub source: Vec<String>,
    pub destination: Vec<String>,
    pub exception: Vec<String>,
}

impl Readconf {
    pub fn new(ficconf: &str) -> Readconf {
        let mut src = Vec::new();
        let mut dst = Vec::new();
        let mut exc = Vec::new();

        let input = File::open(&ficconf);
        match input {
            Err(e) => {
                println!("Error reading conf file {} => {}", &ficconf, e);
            }
            Ok(f) => {
                let buffered = BufReader::new(f);
                for l in buffered.lines().flatten() {
                    //if let Ok(l) = line {
                    let res = get_val(&l);
                    if l.to_lowercase().starts_with("source=") {
                        src.push(res);
                    } else if l.to_lowercase().starts_with("destination=") {
                        dst.push(res);
                    } else if l.to_lowercase().starts_with("exception=") {
                        exc.push(res);
                    }
                    //} //TODO : else ...
                }
            }
        }
        if src.len() != dst.len() {
            panic!("Incorrect Configuration file {}", &ficconf);
        }

        Readconf {
            source: src,
            destination: dst,
            exception: exc,
        }
    }
}

fn get_val(arg: &str) -> String {
    let mut res = String::new();
    for part in arg.split('=').skip(1) {
        if !res.is_empty() {
            res.push('=');
        }
        res.push_str(part);
    }
    res
}
