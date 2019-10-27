mod fic;
pub use fic::*;

use std::ffi::OsString;
use std::path::Path;
use std::collections::HashMap;

pub struct Fold
{
    pub name : OsString,                //folder's name (only it : not the full path!!)
    pub fics : HashMap<OsString,Fic>,   //files inside the folder
    pub folds : HashMap<OsString,Fold>, //sub-filders
    pub forbidden : bool,               //Forbidden folder
}

impl Fold{
    pub fn new_root(dir : &Path)->Fold
    {
        Fold{
            name : (*dir).as_os_str().to_os_string(), 
            fics : HashMap::new(),
            folds : HashMap::new(),
            forbidden : false
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
            forbidden : false
            }
    }

    pub fn add_fold(&mut self, fold : Fold)
    {
        let n = Path::new(&fold.name);
        let name = (n.file_name().unwrap()).to_os_string();
        self.folds.insert(name, fold);
    }

    pub fn add_fic(&mut self, fic : Fic)
    {
        let n = Path::new(&fic.name);
        let name = (n.file_name().unwrap()).to_os_string();
        self.fics.insert(name, fic);
    }

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

    pub fn gen_copy(&self,dst : &Fold)
    {
        println!("searching files/folders to copy to destination");
        let racine_src = Path::new(&self.name);
        let racine_dst = Path::new(&dst.name);
        gen_copy_recurse(&self,&dst,&racine_src,&racine_dst)
    }
 
    pub fn gen_remove(&self,src : &Fold)
    {
        println!("searching files/folders to remove from destination");
        let racine_src = Path::new(&src.name);
        let racine_dst = Path::new(&self.name);
        gen_remove_recurse(&src,&self,&racine_src,&racine_dst)
    }


}

fn gen_copy_recurse(src : &Fold,dst : &Fold,racine_src:&Path,racine_dst:&Path)
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
                let chemin_dst = Path::new(&racine_dst);
                let chemin_src = Path::new(&racine_src);
                let cmd = gen_copy_rec(&chemin_src,&chemin_dst);
                deal_with_cmd(&cmd);
            },
            Some(val_dst) => {
                if val_dst.forbidden 
                {
                    //destination forder is not accessible. Should ignore it
                    println!("{:?}\\{:?} is forbidden -> ignoring",&racine_dst,&key_src);
                    continue;
                }
                //existe en source et destination  -> Ok on procède pour leur contenu
                let new_racine_src = Path::new(&racine_src).join(&key_src);
                let new_racine_dst = Path::new(&racine_dst).join(&key_src);                
                gen_copy_recurse(&val_src,&val_dst,&new_racine_src,&new_racine_dst);
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
                let chemin_src = Path::new(&racine_src).join(&key_src);
                let chemin_dst = Path::new(&racine_dst);
                let cmd = gen_copy(&chemin_src,&chemin_dst);
                deal_with_cmd(&cmd);
            },
            Some(val_dst) => {
                //existe en source et destination  -> les comparer
                if val_src.neq(val_dst)
                {
                    let chemin_src = Path::new(&racine_src).join(&key_src);
                    let chemin_dst = Path::new(&racine_dst);
                    let cmd = gen_copy(&chemin_src,&chemin_dst);
                    deal_with_cmd(&cmd);
                }
            }
        }
    }
}

fn gen_remove_recurse(src : &Fold,dst : &Fold,racine_src:&Path,racine_dst:&Path)
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
                let chemin = Path::new(&racine_dst).join(&key_dst);
                let cmd = gen_rd(&chemin);
                deal_with_cmd(&cmd);
            },
            Some(val_src) => {
                if val_src.forbidden 
                {
                    //source forder is not accessible. Should ignore it
                    println!("{:?}\\{:?} is forbidden -> ignoring",&racine_src,&key_dst);
                    continue;
                }
                //existe en source et destination  -> Ok on procède pour leur contenu
                let new_racine_src = Path::new(&racine_src).join(&key_dst);
                let new_racine_dst = Path::new(&racine_dst).join(&key_dst);                
                gen_remove_recurse(&val_src,&val_dst,&new_racine_src,&new_racine_dst);
            }
        }
    }

    //  -les fics sur destination et pas sur self -> del
    for (key_dst, _val_dst) in dst.fics.iter()
    {
        match src.fics.get(key_dst){
            None => {
                //n'existe pas en destination -> générer un delete
                let chemin = Path::new(&racine_dst).join(&key_dst);
                let cmd = gen_del(&chemin);
                deal_with_cmd(&cmd);
            },
            Some(_) => {
                //existe en source et destination  -> s'ils sont différent c'est le gen_copy qui a généré la copie
            }
        }
    }
    //les Fold des deux côtés -> recurse
}


fn deal_with_cmd(cmd : &OsString)
{
    println!("{:?}",cmd);
}


pub fn gen_copy(src: &Path, dst: &Path)->OsString
{
    let mut res = OsString::new();
    res.push(r###"XCOPY ""###);
    res.push(src);
    res.push(r###"" ""###);
    res.push(dst);
    res.push(r###"" /H /Y /K /R "###);
    // /H   copie aussi les fics cach�s
    // /Y   pas de demande de confirmation
    // /K   copie aussi les attributs
    // /R   remplace les fics lecture seule
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
    // /E   copie les sous-Folds vides
    // /I   destination = r�pertoire si plusieurs fics en sources
    // /H   copie aussi les fics cach�s
    // /Y   pas de demande de confirmation
    // /K   copie aussi les attributs
    // /R   remplace les fics lecture seule
    res
}

pub fn gen_del(dst: &Path)->OsString
{
    let mut res = OsString::new();
    res.push(r###"DEL ""###);
    res.push(dst);
    res.push(r###"" /F /A "###);
    //   /F   force effacement lecture seule   
    //   /A   efface quemque soit les attributs
    res
}

pub fn gen_rd(dst: &Path)->OsString
{
    let mut res = OsString::new();
    res.push(r###"RD /S /Q ""###);
    res.push(dst);
    res.push(r###"""###);
    //   /S   récursif
    //   /Q   pas besoin de confirmation
    res
}
