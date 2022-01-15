use super::constant::*;
use super::fold::*;
use super::logger::*;
use super::paramcli::*;
use super::progression::*;
use super::scriptgen::*;

use std::path::Path;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};
use std::time::Duration;
use std::time::SystemTime;

extern crate chrono;
use chrono::offset::Utc;
use chrono::DateTime;

pub struct Comparer {
    verbose: bool,
    crypt: bool,
    ignore_err: bool,
    to_script: Sender<Command>, //MPSC chanel to send command to be written to output script
    logger: Logger,
    ignore_date_diff: bool,
}

impl Comparer {
    pub fn new(n: String, o: &Options, s: Sender<Command>, to_logger: Sender<String>) -> Comparer {
        let log = Logger::new(n, o.verbose, to_logger);
        Comparer {
            verbose: o.verbose,
            crypt: o.crypt,
            ignore_err: o.ignore_err,
            to_script: s,
            logger: log,
            ignore_date_diff: o.ignore_date_diff,
        }
    }

    pub fn gen_copy(&self, src: &Fold, dst: &Fold) {
        self.logger
            .verbose(format!("Compare to find new/modify from {:?}", &src.name));
        let racine_src = Path::new(&src.name);
        let racine_dst = Path::new(&dst.name);
        let start = SystemTime::now();
        self.gen_copy_recurse(src, dst, racine_src, racine_dst);
        self.logger.timed(
            format!("Finished finding copies from {:?}", &src.name),
            start,
        );
    }

    pub fn gen_remove(&self, src: &Fold, dst: &Fold) {
        self.logger.verbose(format!(
            "Compare to find what to remove from {:?}",
            &dst.name
        ));
        let racine_src = Path::new(&src.name);
        let racine_dst = Path::new(&dst.name);
        let start = SystemTime::now();
        self.gen_remove_recurse(src, dst, racine_src, racine_dst);
        self.logger.timed(
            format!("finished finding what to deletes from {:?}", &dst.name),
            start,
        );
    }

    fn gen_copy_recurse(&self, src: &Fold, dst: &Fold, racine_src: &Path, racine_dst: &Path) {
        //loop on src and destination to find
        //  -folders on src that do not exist on dst => to copy
        //  -folders on both sides -> recuse on content
        for (key_src, val_src) in src.folds.iter() {
            if val_src.forbidden {
                if !self.ignore_err {
                    self.logger.terminating(
                        format!("{:?}\\{:?} is forbidden", &racine_src, &key_src),
                        -1,
                    );
                } else {
                    //src forder is not accessible. Should ignore it
                    self.logger.warning(format!(
                        "{:?}\\{:?} is forbidden -> ignoring",
                        &racine_src, &key_src
                    ));
                }
                continue;
            }
            match dst.folds.get(key_src) {
                None => {
                    //do not exist on dst -> generate recursive copy
                    let chemin_src = Path::new(&racine_src).join(&(val_src.name));
                    let chemin_dst = Path::new(&racine_dst).join(&(val_src.name));
                    self.deal_with_cmd(Command::CopyRecurs(chemin_src, chemin_dst.to_path_buf()));
                }
                Some(val_dst) => {
                    if val_dst.forbidden {
                        if !self.ignore_err {
                            self.logger.terminating(
                                format!("{:?}\\{:?} is forbidden", &racine_src, &key_src),
                                -1,
                            );
                        } else {
                            //dst fold is not accessible. Should ignore it
                            self.logger.warning(format!(
                                "{:?}\\{:?} is forbidden -> ignoring",
                                &racine_dst, &key_src
                            ));
                        }
                        continue;
                    }
                    //exist in both src and dst  -> Ok proceed the content
                    let new_racine_src = Path::new(&racine_src).join(&(val_src.name));
                    let new_racine_dst = Path::new(&racine_dst).join(&(val_src.name));
                    self.gen_copy_recurse(val_src, val_dst, &new_racine_src, &new_racine_dst);
                }
            }
        }
        //  -files on src and not in des should be copied
        //  -files on both sides must be compared (copy if different)
        for (key_src, val_src) in src.fics.iter() {
            match dst.fics.get(key_src) {
                None => {
                    //do not exist on dst  -> gen a copy
                    let chemin_src = Path::new(&racine_src).join(&(val_src.name));
                    let chemin_dst = Path::new(&racine_dst);
                    self.deal_with_cmd(Command::Copy(chemin_src, chemin_dst.to_path_buf()));
                }
                Some(val_dst) => {
                    //TODO: pay attention to autorization
                    //if src forbidden but exist on dst => copy will fail risk to loose dst
                    //if dst forbidden but src autorized => copy will fail
                    //if both forbidden => copy will also probably fail
                    //both exist => must compare
                    let mut same = true;
                    match val_src.comp(val_dst, self.crypt) {
                        FicComp::Same => {
                            same = true;
                        }
                        FicComp::SizeChange(t1, t2) => {
                            same = false;
                            self.logger.verbose(format!(
                                "DEBUG diff {:?} size difference {}/{}",
                                val_src.name, t1, t2
                            ));
                        }
                        FicComp::DateChange(d1, d2) => {
                            if !self.ignore_date_diff {
                                same = false;
                            }
                            if self.verbose {
                                let m1: DateTime<Utc> = d1.into();
                                let m2: DateTime<Utc> = d2.into();
                                self.logger.verbose(format!(
                                    "DEBUG diff    {:?}  date difference {}-{} (raw:{:?}/{:?})",
                                    val_src.name,
                                    m1.format("%d/%m/%Y %T"),
                                    m2.format("%d/%m/%Y %T"),
                                    d1,
                                    d2
                                ));
                            }
                        }
                    }
                    if !same {
                        let chemin_src = Path::new(&racine_src).join(&(val_src.name));
                        let chemin_dst = Path::new(&racine_dst);
                        self.deal_with_cmd(Command::Copy(chemin_src, chemin_dst.to_path_buf()));
                    }
                }
            }
        }
    }

    fn gen_remove_recurse(&self, src: &Fold, dst: &Fold, racine_src: &Path, racine_dst: &Path) {
        //loopp on src and dst  to find
        //  -folder on destination and not on src  -> to remove
        for (key_dst, val_dst) in dst.folds.iter() {
            if val_dst.forbidden {
                if !self.ignore_err {
                    self.logger.terminating(
                        format!("{:?}\\{:?} is forbidden", &racine_dst, &key_dst),
                        -1,
                    );
                } else {
                    //dst fold is not accessible. Should ignore it
                    self.logger.warning(format!(
                        "{:?}\\{:?} is forbidden -> ignoring",
                        &racine_dst, &key_dst
                    ));
                }
                continue;
            }
            match src.folds.get(key_dst) {
                None => {
                    //do not exist on src  -> must remove directory
                    let chemin = Path::new(&racine_dst).join(&(val_dst.name));
                    let d = val_dst.get_counts();
                    self.deal_with_cmd(Command::RemoveFold(chemin, d.0, d.1));
                }
                Some(val_src) => {
                    if val_src.forbidden {
                        if !self.ignore_err {
                            self.logger.terminating(
                                format!("{:?}\\{:?} is forbidden", &racine_src, &key_dst),
                                -1,
                            );
                        } else {
                            //src forder is not accessible. Should ignore it
                            self.logger.warning(format!(
                                "{:?}\\{:?} is forbidden -> ignoring",
                                &racine_src, &key_dst
                            ));
                        }
                        continue;
                    }
                    //exist in both src and dst  -> Ok on procede the content
                    let new_racine_src = Path::new(&racine_src).join(&(val_dst.name));
                    let new_racine_dst = Path::new(&racine_dst).join(&(val_dst.name));
                    self.gen_remove_recurse(val_src, val_dst, &new_racine_src, &new_racine_dst);
                }
            }
        }

        //  -fileson dst but not in src  -> to del
        for (key_dst, val_dst) in dst.fics.iter() {
            match src.fics.get(key_dst) {
                None => {
                    //do not exist on src -> gen a delete
                    let chemin = Path::new(&racine_dst).join(&(val_dst.name));
                    self.deal_with_cmd(Command::RemoveFile(chemin));
                }
                Some(_) => {
                    //exist on src and dst
                    //if different the gen_copy already has managed
                }
            }
        }
    }

    fn deal_with_cmd(&self, cmd: Command) {
        if self.to_script.send(cmd).is_err() {
            self.logger.error("sending command".to_string());
        }
    }
}

/**
 * start a first comparison thread
 * we receive data (2 folders) from a chanel
 * the comparison creates copies commands (for data in source only or in both but with a diff)
 * commands are sent into a output chanel
 * these chanel goes to a thread who is in charge of writing them to outputfile
 */
pub fn start_thread_comp_p(
    from_join: Receiver<(Arc<Fold>, Arc<Fold>)>,
    to_script: Sender<Command>,
    opt: &Options,
    to_logger: Sender<String>,
    to_progress: Sender<Action>,
) -> JoinHandle<()> {
    start_thread_comp(false, from_join, to_script, opt, to_logger, to_progress)
}

/**
 * quite the same as previous thread
 * but generate remove commands for data in destination only
 */
pub fn start_thread_comp_m(
    from_join: Receiver<(Arc<Fold>, Arc<Fold>)>,
    to_script: Sender<Command>,
    opt: &Options,
    to_logger: Sender<String>,
    to_progress: Sender<Action>,
) -> JoinHandle<()> {
    start_thread_comp(true, from_join, to_script, opt, to_logger, to_progress)
}

fn start_thread_comp(
    plus: bool,
    from_join: Receiver<(Arc<Fold>, Arc<Fold>)>,
    to_script: Sender<Command>,
    opt: &Options,
    to_logger: Sender<String>,
    to_progress: Sender<Action>,
) -> JoinHandle<()> {
    let plus = plus;
    let name = if plus { COMPP } else { COMPM };
    let cmp = Comparer::new(name.to_string(), opt, to_script, to_logger);
    spawn(move || {
        let start_elapse = SystemTime::now();
        let mut tps = Duration::new(0, 0);
        cmp.logger.starting();
        let mut nb_comp = 0;
        for (s, d) in from_join {
            let start = SystemTime::now();
            if plus {
                cmp.gen_copy(&s, &d);
            } else {
                cmp.gen_remove(&s, &d);
            }
            nb_comp += 1;
            tps += cmp.logger.timed(format!("Finished one {}", &name), start);
            if to_progress.send(Action::Compare).is_err() {
                cmp.logger.error("error sending to progress".to_string());
                return;
            }
        }
        cmp.logger.dual_timed(
            format!("Finished all {} ({})", &name, nb_comp),
            tps,
            start_elapse,
        );
    })
}
