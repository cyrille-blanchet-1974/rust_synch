use std::time::SystemTime;
use std::ffi::OsString;
use std::path::Path;

pub struct Fic
{
    pub modify : SystemTime,
    pub len : u64,
    pub name : OsString,
}

//File comparison
#[derive(Copy, Clone)]
pub enum FicComp
{
    Same,
    SizeChange(u64,u64),
    DateChange(SystemTime,SystemTime),
}



impl Fic{
    pub fn new(p : &Path)->Option<Fic>
    {
        let m : SystemTime;
        let l : u64;
        let n : OsString;
        n = ((*p).file_name().unwrap()).to_os_string();//why should it failed ?
        let metadata = p.metadata();
        match metadata
        {
            Err(e) =>  {
                println!("Error with metadata of {:?} -> {}",p,e); //appears on files of which i have no access right
                None
                },
            Ok(md) => {
                l = md.len();
                m = md.modified().unwrap(); //What OS doesn't support modification time of a file ?
                Some(Fic{
                    modify : m,
                    len : l,
                    name : n
                    })
            }
        }
    }
/*
    pub fn neq(&self,f : &Fic)->bool    
    {
        if self.len != f.len
        {
            println!("DEBUG diff {:?} différence de taille {}/{}",self.name,self.len,f.len);//TODO: verbos only
            return true;
        }
        /*if self.name != f.name 
        {
            return true;
        }*/
        if self.len != 0 
        {
            if self.modify != f.modify
            {
                let m1: DateTime<Utc> = self.modify.into();
                let m2: DateTime<Utc> = f.modify.into();
                println!("DEBUG diff    {:?} différence de date {}/{}",self.name,m1.format("%d/%m/%Y %T"),m2.format("%d/%m/%Y %T"));//TODO: verbos only
                return true;
            }
        }
        false
    }*/

    pub fn comp(&self,f : &Fic)->FicComp
    {
        if self.len != f.len
        {
            return FicComp::SizeChange(self.len,f.len);
        }
        /*if self.name != f.name 
        {
            return true;
        }*/
        if self.len != 0 
        {
            if self.modify != f.modify
            {
                return FicComp::DateChange(self.modify,f.modify);
            }
        }
        FicComp::Same
    }
}



