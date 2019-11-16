pub use super::fold::*;
pub use super::paramcli::*;

use std::path::Path;
use std::time::SystemTime;
use std::ffi::OsString;
use std::sync::mpsc::Sender;

extern crate chrono;
use chrono::offset::Utc;
use chrono::DateTime;


pub struct Comparer
{
    verbose : bool,
    crypt : bool,
    ignore_err : bool,
    sender : Sender<OsString> //MPSC chanel to send command to be written to output script
}

impl Comparer{
    pub fn new(o : &Options, s : Sender<OsString>)->Comparer
    {
        Comparer
        {
            verbose: o.verbose,
            crypt : o.crypt,
            ignore_err : o.ignore_err,
            sender: s
        }
    }
    
    pub fn gen_copy(&self, src : &Fold, dst : &Fold )
    {
        if self.verbose
        {
            println!("INFO compare to find new/modify in {:?}",&src.name);
        }
        let racine_src = Path::new(&src.name);
        let racine_dst = Path::new(&dst.name);
        let start = SystemTime::now();
        let res = self.gen_copy_recurse(&src,&dst,&racine_src,&racine_dst);
        if self.verbose
        {
            let end = SystemTime::now();
            let tps = end.duration_since(start).expect("ERROR computing duration!");
            println!("INFO duration to find copies from {:?} in {:?}", &src.name, tps);
        }
        res
    }
 
    pub fn gen_remove(&self,src : &Fold, dst : &Fold)
    {
        if self.verbose
        {
            println!("INFO compare to find what to remove from {:?}",&dst.name);
        }
        let racine_src = Path::new(&src.name);
        let racine_dst = Path::new(&dst.name);
        let start = SystemTime::now();
        let res = self.gen_remove_recurse(&src,&dst,&racine_src,&racine_dst);
        if self.verbose
        {
            let end = SystemTime::now();
            let tps = end.duration_since(start).expect("ERROR computing duration!");
            println!("INFO duration to find deletes from {:?} in {:?}", &dst.name, tps);
        }
        res
    }

    fn gen_copy_recurse(&self, src : &Fold,dst : &Fold,racine_src:&Path,racine_dst:&Path)
    {
        //boucle sur self et destination en parallèle et trouve 
        //  -les Folds sur self et pas sur destination -> xcopy
        //  -les Folds des deux cotés -> recurse
        for (key_src, val_src) in src.folds.iter()
        {       
            if val_src.forbidden 
            {
                //source forder is not accessible. Should ignore it
                if self.verbose
                {
                    println!("{:?}\\{:?} is forbidden -> ignoring",&racine_src,&key_src);
                }
                if !self.ignore_err
                {
                    println!("{:?}\\{:?} is forbidden -> stopping",&racine_src,&key_src);
                    std::process::exit(0);
                }
                continue;
            }
            match dst.folds.get(key_src){
                None => {
                    //n'existe pas en destination -> générer une copie récursive
                    let chemin_src = Path::new(&racine_src).join(&(val_src.name));
                    let chemin_dst = Path::new(&racine_dst);
                    let cmd = gen_copy_rec(&chemin_src,&chemin_dst);
                    self.deal_with_cmd(cmd);
                },
                Some(val_dst) => {
                    if val_dst.forbidden 
                    {
                        //destination forder is not accessible. Should ignore it
                        if self.verbose
                        {
                            println!("{:?}\\{:?} is forbidden -> ignoring",&racine_dst,&key_src);
                        }
                        if !self.ignore_err
                        {
                            println!("{:?}\\{:?} is forbidden -> stopping",&racine_src,&key_src);
                            std::process::exit(0);
                        }
                        continue;
                    }
                    //existe en source et destination  -> Ok on procède pour leur contenu
                    let new_racine_src = Path::new(&racine_src).join(&(val_src.name));
                    let new_racine_dst = Path::new(&racine_dst).join(&(val_src.name));
                    self.gen_copy_recurse(&val_src,&val_dst,&new_racine_src,&new_racine_dst);
                }
            }
        }
        //  -les fics sur self et pas en destination -> copy
        //  -les fics des deux côtés -> comparaison et si différent -> copy
        for (key_src, val_src) in src.fics.iter()
        {
            match dst.fics.get(key_src){
                None => {
                    //n'existe pas en destination -> générer une copie
                    let chemin_src = Path::new(&racine_src).join(&(val_src.name));
                    let chemin_dst = Path::new(&racine_dst);
                    let cmd = gen_copy(&chemin_src,&chemin_dst);
                    self.deal_with_cmd(cmd);
                },
                Some(val_dst) => {
                    //existe en source et destination  -> les comparer
                    let mut same = true;
                    match val_src.comp(val_dst,self.crypt)
                    {
                        FicComp::Same => {
                            same=true;
                        },
                        FicComp::SizeChange(t1,t2) => {
                            same= false;
                            if self.verbose
                            {                  
                                println!("DEBUG diff {:?} size difference {}/{}",val_src.name,t1,t2);
                            }
                        },
                        FicComp::DateChange(d1,d2) => {
                            if self.verbose
                            {                   
                                let m1: DateTime<Utc> = d1.into();
                                let m2: DateTime<Utc> = d2.into();
                                println!("DEBUG diff    {:?}  date difference {}-{}",val_src.name,m1.format("%d/%m/%Y %T"),m2.format("%d/%m/%Y %T"));
                            }
                        }
                    }
                    if !same
                    {
                        let chemin_src = Path::new(&racine_src).join(&(val_src.name));
                        let chemin_dst = Path::new(&racine_dst);
                        let cmd = gen_copy(&chemin_src,&chemin_dst);
                        self.deal_with_cmd(cmd);
                    }
                }
            }
        }
    }

    fn gen_remove_recurse(&self, src : &Fold,dst : &Fold,racine_src:&Path,racine_dst:&Path)
    {
        //boucle sur self et source en parallèle et trouve 
        //  -les Folds sur destination et pas sur self -> rd
        for (key_dst, val_dst) in dst.folds.iter()
        {       
            if val_dst.forbidden 
            {
                //destination forder is not accessible. Should ignore it
                if self.verbose
                {                   
                    println!("{:?}\\{:?} is forbidden -> ignoring",&racine_dst,&key_dst);
                }
                if !self.ignore_err
                {
                    println!("{:?}\\{:?} is forbidden -> stopping",&racine_src,&key_dst);
                    std::process::exit(0);
                }
                continue;
            }
            match src.folds.get(key_dst){
                None => {
                    //n'existe pas en destination -> générer un remove directory
                    let chemin = Path::new(&racine_dst).join(&(val_dst.name));
                    let mut cmd = gen_rd(&chemin);
                    let d = val_dst.get_counts();
                    if d.0 > 10 || d.1 > 100
                    {
                        cmd = get_confirmation(&chemin,d,&cmd);
                    }
                    self.deal_with_cmd(cmd);
                },
                Some(val_src) => {
                    if val_src.forbidden 
                    {
                        //source forder is not accessible. Should ignore it
                        if self.verbose
                        {                   
                            println!("{:?}\\{:?} is forbidden -> ignoring",&racine_src,&key_dst);
                        }
                        if !self.ignore_err
                        {
                            println!("{:?}\\{:?} is forbidden -> stopping",&racine_src,&key_dst);
                            std::process::exit(0);
                        }
                        continue;
                    }
                    //existe en source et destination  -> Ok on procède pour leur contenu
                    let new_racine_src = Path::new(&racine_src).join(&(val_dst.name));
                    let new_racine_dst = Path::new(&racine_dst).join(&(val_dst.name));
                    self.gen_remove_recurse(&val_src,&val_dst,&new_racine_src,&new_racine_dst);
                }
            }
        }

        //  -les fics sur destination et pas sur self -> del
        for (key_dst, val_dst) in dst.fics.iter()
        {
            match src.fics.get(key_dst){
                None => {
                    //n'existe pas en destination -> générer un delete
                    let chemin = Path::new(&racine_dst).join(&(val_dst.name));
                    let cmd = gen_del(&chemin);
                    self.deal_with_cmd(cmd);
                },
                Some(_) => {
                    //existe en source et destination  -> s'ils sont différent c'est le gen_copy qui a généré la copie
                }
            }
        }
        //les Fold des deux côtés -> recurse
    }


    fn deal_with_cmd(&self, cmd : OsString)
    {
        if self.sender.send(cmd).is_err()
        {
            println!("Erreur sending command");
        }
    }
}

pub fn gen_copy(src: &Path, dst: &Path)->OsString
{
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

pub fn gen_copy_rec(src: &Path, dst: &Path)->OsString
{
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

pub fn gen_del(dst: &Path)->OsString
{
    let mut res = OsString::new();
    res.push(r###"DEL ""###);
    res.push(dst);
    res.push(r###"" /F /A "###);
    //   /F   Force delete of read only
    //   /A   delete whatever attributes 
    res
}

pub fn gen_rd(dst: &Path)->OsString
{
    let mut res = OsString::new();
    res.push(r###"RD /S /Q ""###);
    res.push(dst);
    res.push(r###"""###);
    //   /S   recursive
    //   /Q   No confirmation ask to user
    res
}

pub fn get_confirmation(dst: &Path, c:(u32,u32), cmd : &OsString)->OsString
{
    let mut res = OsString::new();
    let s = format!("Echo {:?} Contains {} folders and {}  files.\n",dst,c.0,c.1);
    res.push(s);
    /*res.push("Echo ");
    res.push(dst);
    res.push("Contains ");
    res.push(c.0);
    res.push(" folders and ");
    res.push(c.1);
    res.push(" files.\n");*/
    res.push("Echo Please confirm deletation\n");
    res.push("Echo Y to Delete\n");
    res.push("Echo N to keep\n");
    res.push("choice /C YN\n");
    res.push("if '%ERRORLEVE%'=='1' ");
    res.push(cmd);
    res
}
