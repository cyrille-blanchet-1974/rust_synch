use std::time::SystemTime;
use std::ffi::OsString;
use std::path::Path;

pub struct Fic
{
    pub modify : SystemTime,
    pub len : u64,
    pub name : OsString,
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

    pub fn neq(&self,f : &Fic)->bool    
    {
        if self.len != f.len
        {
            return false;
        }
        if self.name != f.name 
        {
            return false;
        }
        if self.modify != f.modify
        {
            return false;
        }
        true
    }
}
