use super::constant::*;
use super::logger::*;
use super::writer::*;

use std::boxed::Box;
use std::ffi::OsString;
use std::path::Path;
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

#[cfg(windows)]
pub fn gen_copy(src: &Path, dst: &Path) -> OsString {
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
#[cfg(unix)]
pub fn gen_copy(src: &Path, dst: &Path) -> OsString {
    let mut res = OsString::new();
    res.push(r###"cp --preserve=all ""###);
    res.push(src);
    res.push(r###"" ""###);
    res.push(dst);
    res.push(r###"" "###);
    // --preseve=all copy attributes ownership datetime
    res
}

#[cfg(windows)]
pub fn gen_copy_rec(src: &Path, dst: &Path) -> OsString {
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
#[cfg(unix)]
pub fn gen_copy_rec(src: &Path, dst: &Path) -> OsString {
    let mut res = OsString::new();
    //linux recursive copy : cp --preserve=all -r src/ dst
    res.push(r###"cp --preserve=all -r ""###);
    res.push(src);
    res.push("/");
    res.push(r###"" ""###);
    res.push(dst);
    res.push(r###"" "###);
    // --preseve=all copy attributes ownership datetime
    // -r             recursive
    res
}

#[cfg(windows)]
pub fn gen_del(dst: &Path) -> OsString {
    let mut res = OsString::new();
    res.push(r###"DEL ""###);
    res.push(dst);
    res.push(r###"" /F /A "###);
    //   /F   Force delete of read only
    //   /A   delete whatever attributes
    res
}
#[cfg(unix)]
pub fn gen_del(dst: &Path) -> OsString {
    let mut res = OsString::new();
    //linux remove rm -f fic
    res.push(r###"rm -f ""###);
    res.push(dst);
    res.push(r###"" "###);
    //   -f   Force delete
    res
}

#[cfg(windows)]
pub fn gen_rd(dst: &Path, nbfic: u32, nbfold: u32) -> OsString {
    let mut res = OsString::new();
    if nbfold > 10 || nbfic > 100 {
        let s = format!(
            "Echo {:?} Contains {} folders and {}  files.",
            dst, nbfold, nbfic
        );
        //check shell command for asking
        res.push(s);
        res.push(EOL);
        res.push("Echo Please confirm deletation");
        res.push(EOL);
        res.push("Echo Y to Delete");
        res.push(EOL);
        res.push("Echo N to keep");
        res.push(EOL);
        res.push("choice /C YN");
        res.push(EOL);
        res.push("if '%ERRORLEVEL%'=='1' ");
    }
    res.push(r###"RD /S /Q ""###);
    res.push(dst);
    res.push(r###"""###);
    //   /S   recursive
    //   /Q   No confirmation ask to user
    res
}
#[cfg(unix)]
pub fn gen_rd(dst: &Path, nbfic: u32, nbfold: u32) -> OsString {
    let mut cmd = OsString::new();
    cmd.push(r###"rm -rf ""###);
    cmd.push(dst);
    cmd.push(r###"""###);
    //   -rf  recursive and force
    if nbfold > 10 || nbfic > 100 {
        let mut res = OsString::new();
        let s = format!(
            "echo {:?} Contains {} folders and {}  files.",
            dst, nbfold, nbfic
        );
        res.push(s);
        res.push(EOL);
        res.push("echo Do you wish to delete this folder? \"Yes\" will Proceed, Anything Else will keep it");
        res.push(EOL);
        res.push("    read yn");
        res.push(EOL);
        res.push("        case $yn in");
        res.push(EOL);
        res.push("            Yes) ");
        res.push(cmd);
        res.push(";;");
        res.push(EOL);
        res.push("            *) echo avoid delete ;;");
        res.push(EOL);
        res.push("        esac");
        res.push(EOL);
        res
    } else {
        cmd
    }
}

#[cfg(unix)]
pub fn start_script() -> (String, String) {
    ("#/bin/sh".to_string(), "".to_string())
}

#[cfg(windows)]
pub fn start_script() -> (String, String) {
    ("@echo off".to_string(), "chcp 65001".to_string())
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
        let begin = start_script();
        writer.write(begin.0);
        writer.write(begin.1);
        //linux; start with #/usr/bin/sh
        //writer.write("@echo off".to_string());
        //writer.write("chcp 65001".to_string()); //utf8 codepage
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
