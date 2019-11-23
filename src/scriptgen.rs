use super::writer::*;

use std::boxed::Box;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, SystemTime};

pub enum Command {
    Copy(PathBuf, PathBuf),
    CopyRecurs(PathBuf, PathBuf),
    RemoveFile(PathBuf),
    RemoveFold(PathBuf, u32, u32),
}

impl Command {
    pub fn to_command(&self) -> OsString {
        match self {
            Command::Copy(src, dst) => gen_copy(src, dst),
            Command::CopyRecurs(src, dst) => gen_copy_rec(src, dst),
            Command::RemoveFile(dst) => gen_del(dst),
            Command::RemoveFold(dst, nbfold, nbfic) => gen_rd(dst, nbfold, nbfic),
        }
    }
}

pub fn gen_copy(src: &PathBuf, dst: &PathBuf) -> OsString {
    let mut res = OsString::new();
    res.push(r###"XCOPY ""###);
    res.push(src);
    res.push(r###"" ""###);
    res.push(dst);
    res.push(r###"" /H /Y /K /R "###);
    // /H   also copy hidden files
    // /Y   No confirmation ask to user
    // /K   copy attributes
    // /R   replace Read only files
    res
}

pub fn gen_copy_rec(src: &PathBuf, dst: &PathBuf) -> OsString {
    let mut res = OsString::new();
    res.push(r###"XCOPY ""###);
    res.push(src);
    res.push("\\*.*");
    res.push(r###"" ""###);
    res.push(dst);
    res.push(r###"" /E /I /H /Y /K /R "###);
    // /E   copy empty sub folders
    // /I   choose folder as destination if many files in source
    // /H   also copy hidden files
    // /Y   No confirmation ask to user
    // /K   copy attributes
    // /R   replace Read only files
    res
}

pub fn gen_del(dst: &PathBuf) -> OsString {
    let mut res = OsString::new();
    res.push(r###"DEL ""###);
    res.push(dst);
    res.push(r###"" /F /A "###);
    //   /F   Force delete of read only
    //   /A   delete whatever attributes
    res
}

pub fn gen_rd(dst: &PathBuf, nbfic: &u32, nbfold: &u32) -> OsString {
    let mut res = OsString::new();
    if *nbfold > 10 || *nbfic > 100 {
        let s = format!(
            "Echo {:?} Contains {} folders and {}  files.\n",
            dst, nbfold, nbfic
        );
        res.push(s);
        res.push("Echo Please confirm deletation\n");
        res.push("Echo Y to Delete\n");
        res.push("Echo N to keep\n");
        res.push("choice /C YN\n");
        res.push("if '%ERRORLEVE%'=='1' ");
    }
    res.push(r###"RD /S /Q ""###);
    res.push(dst);
    res.push(r###"""###);
    //   /S   recursive
    //   /Q   No confirmation ask to user
    res
}

fn log(data: String, to_logger: &Sender<String>) {
    if to_logger.send(data).is_err() {
        println!("Erreur sending log");
    }
}

/**
* start output thread
* we open/create the destination file
* for each command receive from the chanel we write in output
*
*/
pub fn start_thread_writer(
    from_comp: Receiver<Command>,
    output: PathBuf,
    to_logger: Sender<String>,
) -> JoinHandle<()> {
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
        log("INFO start writer".to_string(), &to_logger);
        writer.as_mut().write("@echo off".to_string());
        writer.as_mut().write("chcp 65001".to_string()); //utf8 codepage
        for data in from_comp {
            let start = SystemTime::now();
            writer
                .as_mut()
                .write(data.to_command().to_str().unwrap().to_string());
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
        log(
            format!(
                "INFO {} lignes writes in {:?}/{:?}",
                nb_ecr, tps, tps_elapse
            )
            .to_string(),
            &to_logger,
        );
    });
    handle
}
