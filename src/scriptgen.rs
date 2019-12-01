use super::constant::*;
use super::logger::*;
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
            Command::RemoveFold(dst, nbfold, nbfic) => gen_rd(dst, *nbfold, *nbfic),
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

pub fn gen_rd(dst: &PathBuf, nbfic: u32, nbfold: u32) -> OsString {
    let mut res = OsString::new();
    if nbfold > 10 || nbfic > 100 {
        let s = format!(
            "Echo {:?} Contains {} folders and {}  files.\n",
            dst, nbfold, nbfic
        );
        res.push(s);
        res.push("Echo Please confirm deletation\n");
        res.push("Echo Y to Delete\n");
        res.push("Echo N to keep\n");
        res.push("choice /C YN\n");
        res.push("if '%ERRORLEVEL%'=='1' ");
    }
    res.push(r###"RD /S /Q ""###);
    res.push(dst);
    res.push(r###"""###);
    //   /S   recursive
    //   /Q   No confirmation ask to user
    res
}

/**
* start output thread
* we open/create the destination file
* for each command receive from the chanel we write in output
*
*/
pub fn start_thread_scriptgen(
    from_comp: Receiver<Command>,
    output: PathBuf,
    to_logger: Sender<String>,
    verbose: bool,
) -> JoinHandle<()> {
    spawn(move || {
        let logger = Logger::new(SCRIPTGEN.to_string(), verbose, to_logger);
        logger.starting();
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
        //unbox -> get a pointer to either a WriterDisk or a WriterConsole (anything witch implement Writing trait)
        let writer = writer.as_mut(); //redifinig writer to remove all .as_mut()
        let start_elapse = SystemTime::now();
        let mut tps = Duration::new(0, 0);
        writer.write("@echo off".to_string());
        writer.write("chcp 65001".to_string()); //utf8 codepage
        for data in from_comp {
            let start = SystemTime::now();
            let c = data.to_command();
            let cmd = match c.to_str() {
                Some(s) => s,
                None => {
                    logger.terminating(format!("Non unicode characters in {:?}", c), -4);
                    return; //dead code beacause terminating stop the prg but compiler don't know that
                }
            };
            writer.write(cmd.to_string());
            let end = SystemTime::now();
            tps += end
                .duration_since(start)
                .expect("ERROR computing duration!");
            //no log here (too much too write a log for each command written to script)
        }
        let nb_ecr = writer.get_nb_ecr();
        logger.dual_timed(
            format!("Finished: {} lines writens", nb_ecr),
            tps,
            start_elapse,
        );
    })
}
