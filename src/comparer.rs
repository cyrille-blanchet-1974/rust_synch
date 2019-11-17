pub use super::fold::*;
pub use super::paramcli::*;
pub use super::scriptgen::*;
pub use super::logger::*;

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
}

impl Comparer {
    pub fn new(o: &Options, s: Sender<Command>,to_logger: Sender<String>) -> Comparer {
        let log = Logger::new(o.verbose,to_logger);
        Comparer {
            verbose: o.verbose,
            crypt: o.crypt,
            ignore_err: o.ignore_err,
            to_script: s,
            logger: log,
        }
    }

    pub fn gen_copy(&self, src: &Fold, dst: &Fold) {
        self.logger.log_verbose(format!("INFO compare to find new/modify in {:?}", &src.name));
        let racine_src = Path::new(&src.name);
        let racine_dst = Path::new(&dst.name);
        let start = SystemTime::now();
        let res = self.gen_copy_recurse(&src, &dst, &racine_src, &racine_dst);
        self.logger.log_timed_verbose(format!("INFO duration to find copies from {:?} ",&src.name), start);
        res
    }

    pub fn gen_remove(&self, src: &Fold, dst: &Fold) {
        self.logger.log_verbose(format!("INFO compare to find what to remove from {:?}", &dst.name));
        let racine_src = Path::new(&src.name);
        let racine_dst = Path::new(&dst.name);
        let start = SystemTime::now();
        let res = self.gen_remove_recurse(&src, &dst, &racine_src, &racine_dst);
        self.logger.log_timed_verbose(format!("INFO duration to find deletes from {:?}",&dst.name), start);
        res
    }

    fn gen_copy_recurse(&self, src: &Fold, dst: &Fold, racine_src: &Path, racine_dst: &Path) {
        //boucle sur self et destination en parallèle et trouve
        //  -les Folds sur self et pas sur destination -> xcopy
        //  -les Folds des deux cotés -> recurse
        for (key_src, val_src) in src.folds.iter() {
            if val_src.forbidden {
                //source forder is not accessible. Should ignore it
                self.logger.log_verbose(format!("{:?}\\{:?} is forbidden -> ignoring", &racine_src, &key_src));
                if !self.ignore_err {
                    println!("{:?}\\{:?} is forbidden -> stopping", &racine_src, &key_src);
                    std::process::exit(0);
                }
                continue;
            }
            match dst.folds.get(key_src) {
                None => {
                    //n'existe pas en destination -> générer une copie récursive
                    let chemin_src = Path::new(&racine_src).join(&(val_src.name));
                    let chemin_dst = Path::new(&racine_dst);
                    self.deal_with_cmd(Command::CopyRecurs(chemin_src, chemin_dst.to_path_buf()));
                }
                Some(val_dst) => {
                    if val_dst.forbidden {
                        //destination forder is not accessible. Should ignore it
                        self.logger.log_verbose(format!("{:?}\\{:?} is forbidden -> ignoring", &racine_dst, &key_src));
                        if !self.ignore_err {
                            println!("{:?}\\{:?} is forbidden -> stopping", &racine_src, &key_src);
                            std::process::exit(0);
                        }
                        continue;
                    }
                    //existe en source et destination  -> Ok on procède pour leur contenu
                    let new_racine_src = Path::new(&racine_src).join(&(val_src.name));
                    let new_racine_dst = Path::new(&racine_dst).join(&(val_src.name));
                    self.gen_copy_recurse(&val_src, &val_dst, &new_racine_src, &new_racine_dst);
                }
            }
        }
        //  -les fics sur self et pas en destination -> copy
        //  -les fics des deux côtés -> comparaison et si différent -> copy
        for (key_src, val_src) in src.fics.iter() {
            match dst.fics.get(key_src) {
                None => {
                    //n'existe pas en destination -> générer une copie
                    let chemin_src = Path::new(&racine_src).join(&(val_src.name));
                    let chemin_dst = Path::new(&racine_dst);
                    self.deal_with_cmd(Command::Copy(chemin_src, chemin_dst.to_path_buf()));
                }
                Some(val_dst) => {
                    //existe en source et destination  -> les comparer
                    let mut same = true;
                    match val_src.comp(val_dst, self.crypt) {
                        FicComp::Same => {
                            same = true;
                        }
                        FicComp::SizeChange(t1, t2) => {
                            same = false;
                            self.logger.log_verbose(format!("DEBUG diff {:?} size difference {}/{}",val_src.name, t1, t2));
                        }
                        FicComp::DateChange(d1, d2) => {
                            if self.verbose {
                                let m1: DateTime<Utc> = d1.into();
                                let m2: DateTime<Utc> = d2.into();
                                self.logger.log_verbose(format!("DEBUG diff    {:?}  date difference {}-{}",val_src.name,m1.format("%d/%m/%Y %T"),m2.format("%d/%m/%Y %T")));
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
        //boucle sur self et source en parallèle et trouve
        //  -les Folds sur destination et pas sur self -> rd
        for (key_dst, val_dst) in dst.folds.iter() {
            if val_dst.forbidden {
                //destination forder is not accessible. Should ignore it
                self.logger.log_verbose(format!("{:?}\\{:?} is forbidden -> ignoring", &racine_dst, &key_dst));
                if !self.ignore_err {
                    println!("{:?}\\{:?} is forbidden -> stopping", &racine_src, &key_dst);
                    std::process::exit(0);
                }
                continue;
            }
            match src.folds.get(key_dst) {
                None => {
                    //n'existe pas en destination -> générer un remove directory
                    let chemin = Path::new(&racine_dst).join(&(val_dst.name));
                    let d = val_dst.get_counts();
                    self.deal_with_cmd(Command::RemoveFold(chemin, d.0, d.1));
                }
                Some(val_src) => {
                    if val_src.forbidden {
                        //source forder is not accessible. Should ignore it
                        self.logger.log_verbose(format!("{:?}\\{:?} is forbidden -> ignoring", &racine_src, &key_dst));
                        if !self.ignore_err {
                            println!("{:?}\\{:?} is forbidden -> stopping", &racine_src, &key_dst);
                            std::process::exit(0);
                        }
                        continue;
                    }
                    //existe en source et destination  -> Ok on procède pour leur contenu
                    let new_racine_src = Path::new(&racine_src).join(&(val_dst.name));
                    let new_racine_dst = Path::new(&racine_dst).join(&(val_dst.name));
                    self.gen_remove_recurse(&val_src, &val_dst, &new_racine_src, &new_racine_dst);
                }
            }
        }

        //  -les fics sur destination et pas sur self -> del
        for (key_dst, val_dst) in dst.fics.iter() {
            match src.fics.get(key_dst) {
                None => {
                    //n'existe pas en destination -> générer un delete
                    let chemin = Path::new(&racine_dst).join(&(val_dst.name));
                    //let cmd = gen_del(&chemin);
                    self.deal_with_cmd(Command::RemoveFile(chemin));
                }
                Some(_) => {
                    //existe en source et destination  -> s'ils sont différent c'est le gen_copy qui a généré la copie
                }
            }
        }
        //les Fold des deux côtés -> recurse
    }

    fn deal_with_cmd(&self, cmd: Command) {
        if self.to_script.send(cmd).is_err() {
            println!("Erreur sending command");
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
    to_logger: Sender<String>
) -> JoinHandle<()> {
    /*
    let cmp = Comparer::new(opt, to_script, to_logger);
    let handle = spawn(move || {
        let start_elapse = SystemTime::now();
        let mut tps = Duration::new(0, 0);
        cmp.log("INFO start comp_p".to_string());
        let mut nb_comp = 0;
        for (s, d) in from_join {
            let start = SystemTime::now();
            cmp.gen_copy(&s, &d);
            nb_comp += 1;
            let end = SystemTime::now();
            tps += end
                .duration_since(start)
                .expect("ERROR computing duration!");
        }
        let end_elapse = SystemTime::now();
        let tps_elapse = end_elapse
            .duration_since(start_elapse)
            .expect("ERROR computing duration!");
        cmp.log(format!("INFO {} comp_p in {:?}/{:?}", nb_comp, tps, tps_elapse));
    });
    handle*/
    start_thread_comp(false,from_join,to_script,opt,to_logger)
}

/**
 * quite the same as previous thread
 * but generate remove commands for data in destination only
 */
pub fn start_thread_comp_m(
    from_join: Receiver<(Arc<Fold>, Arc<Fold>)>,
    to_script: Sender<Command>,
    opt: &Options,
    to_logger: Sender<String>
) -> JoinHandle<()> {
    /*
    let cmp = Comparer::new(opt, to_script,to_logger);
    let handle = spawn(move || {
        let start_elapse = SystemTime::now();
        let mut tps = Duration::new(0, 0);
        cmp.log("INFO start comp_m".to_string());
        let mut nb_comp = 0;
        for (s, d) in from_join {
            let start = SystemTime::now();
            cmp.gen_remove(&s, &d);
            nb_comp += 1;
            let end = SystemTime::now();
            tps += end
                .duration_since(start)
                .expect("ERROR computing duration!");
        }
        let end_elapse = SystemTime::now();
        let tps_elapse = end_elapse
            .duration_since(start_elapse)
            .expect("ERROR computing duration!");
        cmp.log(format!("INFO {} comp_m in {:?}/{:?}", nb_comp, tps, tps_elapse));
    });
    handle*/
    start_thread_comp(true,from_join,to_script,opt,to_logger)
}

fn start_thread_comp(
	plus : bool,
    from_join: Receiver<(Arc<Fold>, Arc<Fold>)>,
    to_script: Sender<Command>,
    opt: &Options,
    to_logger: Sender<String>
) -> JoinHandle<()> {
	let plus = plus;
	let name;
	if plus
	{
		name = "comp_p";
	}
	else
	{
		name = "comp_m";
	}
    let cmp = Comparer::new(opt, to_script, to_logger);
    let handle = spawn(move || {
        let start_elapse = SystemTime::now();
        let mut tps = Duration::new(0, 0);
        cmp.logger.log(format!("INFO start {}",&name));
        let mut nb_comp = 0;
        for (s, d) in from_join {
            let start = SystemTime::now();
			if plus
			{
				cmp.gen_copy(&s, &d);
			}
			else
			{
				cmp.gen_remove(&s, &d);
			}            
            nb_comp += 1;
            let end = SystemTime::now();
            tps += end
                .duration_since(start)
                .expect("ERROR computing duration!");
        }
        let end_elapse = SystemTime::now();
        let tps_elapse = end_elapse
            .duration_since(start_elapse)
            .expect("ERROR computing duration!");
        cmp.logger.log(format!("INFO {} {} in {:?}/{:?}", &name, nb_comp, tps, tps_elapse));
    });
    handle
}
