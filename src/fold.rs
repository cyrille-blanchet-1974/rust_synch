pub use super::fic::*;

use std::ffi::OsString;
use std::path::Path;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::time::SystemTime;

extern crate chrono;
use chrono::offset::Utc;
use chrono::DateTime;

pub struct Fold
{
    pub name : OsString,                //folder's name (only it : not the full path!!)
    pub fics : HashMap<OsString,Fic>,   //files inside the folder
    pub folds : HashMap<OsString,Fold>, //sub-filders
    pub forbidden : bool,               //Forbidden folder
    pub folder_count: u32,              //number of subfolder (recursively)
    pub file_count: u32,                //number of files (recursively)
}

impl Fold{
    pub fn new_root(dir : &Path)->Fold
    {
        Fold{
            name : (*dir).as_os_str().to_os_string(), 
            fics : HashMap::new(),
            folds : HashMap::new(),
            forbidden : false,
            folder_count: 0,
            file_count: 0
            }
    }

    pub fn new(dir : &Path)->Fold
    {
        let n = 
        match (*dir).file_name()
        {
            Some(n) => n.to_os_string(), // pour un Fold ou un fic quelconque => son nom
            None => (*dir).as_os_str().to_os_string(),  //pour / ou c:\ ...   on garde tout
        };
        Fold{
            name : n, 
            fics : HashMap::new(),
            folds : HashMap::new(),
            forbidden : false,
            folder_count: 0,
            file_count: 0
            }
    }

    pub fn add_fold(&mut self, fold : Fold)
    {
        let n = Path::new(&fold.name);
        let name = (n.file_name().unwrap()).to_os_string();
        self.folder_count +=1;
        let d = fold.get_counts();
        self.folder_count += d.0; 
        self.file_count += d.1;
        self.folds.insert(to_lower(&name), fold);
    }

    pub fn add_fic(&mut self, fic : Fic)
    {
        let n = Path::new(&fic.name);
        let name = (n.file_name().unwrap()).to_os_string();
        self.file_count +=1;
        self.fics.insert(to_lower(&name), fic);
    }

    pub fn get_counts(&self)->(u32,u32)
    {
        (self.folder_count,self.file_count)
    }

//non utilisé...
/*
    pub fn display(&self)
    {
        let nouvelle_racine = Path::new(&self.name);
        self.display_recurse(&nouvelle_racine);
    }    

    fn display_recurse(&self,racine:&Path)
    {
        let nouvelle_racine = Path::new(racine).join(&self.name);
        println!("{:?}",&nouvelle_racine);
        for (_key, val) in self.fics.iter()
        {
            let nouveau_fic = Path::new(&nouvelle_racine).join(&val.name);
            println!("{:?}",nouveau_fic);
        }
        for (_key, val) in self.folds.iter()
        {
            val.display_recurse(&nouvelle_racine);
        }
    }    
*/
    pub fn gen_copy(&self,dst : &Fold,sender : &Sender<OsString>)
    {
        println!("INFO compare to find new/modify in {:?}",&self.name);
        let racine_src = Path::new(&self.name);
        let racine_dst = Path::new(&dst.name);
        let start = SystemTime::now();
        let res = gen_copy_recurse(&self,&dst,&racine_src,&racine_dst,sender);
        let end = SystemTime::now();
        let tps = end.duration_since(start).expect("ERROR computing duration!");
        println!("INFO duration to find copies from {:?} in {:?}", &self.name, tps);
        res

    }
 
    pub fn gen_remove(&self,src : &Fold,sender : &Sender<OsString>)
    {
        println!("INFO compare to find what to remove from {:?}",&self.name);
        println!("INFO searching files/folders to remove from destination");
        let racine_src = Path::new(&src.name);
        let racine_dst = Path::new(&self.name);
        let start = SystemTime::now();
        let res = gen_remove_recurse(&src,&self,&racine_src,&racine_dst,sender);
        let end = SystemTime::now();
        let tps = end.duration_since(start).expect("ERROR computing duration!");
        println!("INFO duration to find deletes from {:?} in {:?}", &self.name, tps);
        res
    }


}

fn gen_copy_recurse(src : &Fold,dst : &Fold,racine_src:&Path,racine_dst:&Path,sender : &Sender<OsString>)
{
    //boucle sur self et destination en parallèle et trouve 
    //  -les Folds sur self et pas sur destination -> xcopy
    //  -les Folds des deux cotés -> recurse
    for (key_src, val_src) in src.folds.iter()
    {       
        if val_src.forbidden 
        {
            //source forder is not accessible. Should ignore it
            println!("{:?}\\{:?} is forbidden -> ignoring",&racine_src,&key_src);
            continue;
        }
        match dst.folds.get(key_src){
            None => {
                //n'existe pas en destination -> générer une copie récursive
                let chemin_src = Path::new(&racine_src).join(&(val_src.name));
                let chemin_dst = Path::new(&racine_dst);
                let cmd = gen_copy_rec(&chemin_src,&chemin_dst);
                deal_with_cmd(cmd,sender);
            },
            Some(val_dst) => {
                if val_dst.forbidden 
                {
                    //destination forder is not accessible. Should ignore it
                    println!("{:?}\\{:?} is forbidden -> ignoring",&racine_dst,&key_src);
                    continue;
                }
                //existe en source et destination  -> Ok on procède pour leur contenu
                let new_racine_src = Path::new(&racine_src).join(&(val_src.name));
                let new_racine_dst = Path::new(&racine_dst).join(&(val_src.name));
                gen_copy_recurse(&val_src,&val_dst,&new_racine_src,&new_racine_dst,sender);
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
                deal_with_cmd(cmd,sender);
            },
            Some(val_dst) => {
                //existe en source et destination  -> les comparer
                let mut same = true;
                match val_src.comp(val_dst)
                {
                    FicComp::Same => {
                        same=true;
                    },
                    FicComp::SizeChange(t1,t2) => {
                        same= false;
                        //TODO: verbose only
                        println!("DEBUG diff {:?} différence de taille {}/{}",val_src.name,t1,t2);
                    },
                    FicComp::DateChange(d1,d2) => {
                        let m1: DateTime<Utc> = d1.into();
                        let m2: DateTime<Utc> = d2.into();
                        //TODO: verbose only
                        println!("DEBUG diff    {:?} différence de date {}/{}",val_src.name,m1.format("%d/%m/%Y %T"),m2.format("%d/%m/%Y %T"));
                    }
                }
                if !same
                {
                    let chemin_src = Path::new(&racine_src).join(&(val_src.name));
                    let chemin_dst = Path::new(&racine_dst);
                    let cmd = gen_copy(&chemin_src,&chemin_dst);
                    deal_with_cmd(cmd,sender);
                }
            }
        }
    }
}

fn gen_remove_recurse(src : &Fold,dst : &Fold,racine_src:&Path,racine_dst:&Path,sender : &Sender<OsString>)
{
    //boucle sur self et source en parallèle et trouve 
    //  -les Folds sur destination et pas sur self -> rd
    for (key_dst, val_dst) in dst.folds.iter()
    {       
        if val_dst.forbidden 
        {
            //destination forder is not accessible. Should ignore it
            println!("{:?}\\{:?} is forbidden -> ignoring",&racine_dst,&key_dst);
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
                deal_with_cmd(cmd,sender);
            },
            Some(val_src) => {
                if val_src.forbidden 
                {
                    //source forder is not accessible. Should ignore it
                    println!("{:?}\\{:?} is forbidden -> ignoring",&racine_src,&key_dst);
                    continue;
                }
                //existe en source et destination  -> Ok on procède pour leur contenu
                let new_racine_src = Path::new(&racine_src).join(&(val_dst.name));
                let new_racine_dst = Path::new(&racine_dst).join(&(val_dst.name));
                gen_remove_recurse(&val_src,&val_dst,&new_racine_src,&new_racine_dst,sender);
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
                deal_with_cmd(cmd,sender);
            },
            Some(_) => {
                //existe en source et destination  -> s'ils sont différent c'est le gen_copy qui a généré la copie
            }
        }
    }
    //les Fold des deux côtés -> recurse
}


fn deal_with_cmd(cmd : OsString,sender : &Sender<OsString>)
{
    if sender.send(cmd).is_err()
    {
        println!("Erreur sending command");
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

pub fn to_lower(name : &OsString)->OsString
{
    let res : OsString;
    let clone = OsString::from(name);
    match clone.into_string()
    {
        Ok(a) =>{
            let a = a.to_lowercase();
            res=OsString::from(a);
        },
        Err(a)=>{
            res=a;
        }
    }
    //println!("{:?}=>{:?}",&name,&res);
    return res;
}
