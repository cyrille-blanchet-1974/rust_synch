use std::time::SystemTime;
use std::ffi::OsString;
use std::path::Path;

pub struct Fichier
{
    modify : SystemTime,
    len : u64,
    name : OsString,
}

impl Fichier{
    pub fn new(p : &Path)->Option<Fichier>
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
                Some(Fichier{
                    modify : m,
                    len : l,
                    name : n
                    })
            }
        }
    }
}
