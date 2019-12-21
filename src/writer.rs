use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

//trait implementation for write
pub trait Writing {
    fn write(&mut self, data: String);
    fn get_nb_ecr(&self) -> i32;
}

pub struct WriterDisk {
    nb_ecr: i32,
    writer: BufWriter<File>,
}

// write on disk
impl WriterDisk {
    pub fn new(output: PathBuf, create: bool) -> Option<WriterDisk> {
        let file = if create {
            match File::create(output)//append ?
            {
                    Err(e) =>{
                        println!("Error creating file {:?}", e);
                        return None;
                    },
                    Ok(fil) =>
                    {
                        fil
                    }
            }
        } else {
            match OpenOptions::new().append(true).open(output) {
                Err(e) => {
                    println!("Error opening file {:?} for writing", e);
                    return None;
                }
                Ok(fil) => fil,
            }
        };
        Some(WriterDisk {
            nb_ecr: 0,
            writer: BufWriter::new(file),
        })
    }
}

#[cfg(unix)]
pub static EOL: &str = "\n";

#[cfg(windows)]
pub static EOL: &str = "\r\n";

impl Writing for WriterDisk {
    //how to write a log to disk
    fn write(&mut self, data: String) {
        let data2 = format!("{}{}", data,EOL);
        match self.writer.write_all(data2.as_bytes()) {
            Err(e) => {
                println!("Error writing in file {:?}", e);
            }
            Ok(_) => {
                self.nb_ecr += 1;
            }
        }
    }
    //how many logs writtens
    fn get_nb_ecr(&self) -> i32 {
        self.nb_ecr
    }
}

//write to console
pub struct WriterConsole {
    pub nb_ecr: i32,
}

impl WriterConsole {
    pub fn new() -> WriterConsole {
        WriterConsole { nb_ecr: 0 }
    }
}

impl Writing for WriterConsole {
    //how to write to console
    fn write(&mut self, data: String) {
        println!("{}", data);
        self.nb_ecr += 1;
    }
    //how many logs writtens
    fn get_nb_ecr(&self) -> i32 {
        self.nb_ecr
    }
}
