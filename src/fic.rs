use std::ffi::OsString;
use std::path::Path;
use std::time::SystemTime;

//File properties
pub struct Fic {
    pub forbidden: bool,                //Forbidden folder
    pub modify: SystemTime,             //modification date
    pub len: u64,                       //length
    pub name: OsString,                 //name
}

//File comparison results
#[derive(Copy, Clone)]
pub enum FicComp {
    Same,
    SizeChange(u64, u64),
    DateChange(SystemTime, SystemTime),
}

impl Fic {
    //create a new 'file' in memory
    pub fn new(p: &Path) -> Option<Fic> {
        let m: SystemTime; //last modification date of the file
        let l: u64;        //length of the file
        let n: OsString;   //name of the file
        let mut f = false;
        match (*p).file_name() {
            None => {
                //should only fail if path is a folder and not a file...
                println!("This error should only appear while developping !!! {:?} is a folder and not a file!",p);
                std::process::exit(-2);
            },
            Some(name) => { n= name.to_os_string();}
        }
        //read file infos
        match p.metadata() {
            Err(e) => {
                println!("Error with metadata of {:?} -> {}", p, e); //appears on files of which i have no access right
                f=true;
                l=0;
                m=SystemTime::now();
            }
            Ok(md) => {
                l = md.len();
                m = match md.modified() {
                    Err(e) => {
                        println!("It seems that your filesystem does not support modification time! It's odd. You should probably not use this prog on this system => {}",e);
                        std::process::exit(-3);
                    }
                    Ok(data) => {
                        data
                    }
                };
            }
        }
        //we have all we need to produce our File struct
        Some(Fic {
            forbidden : f,
            modify: m,
            len: l,
            name: n,
        })

    }

    pub fn comp(&self, f: &Fic, crypt: bool) -> FicComp {
        if crypt {
            //host use crypting
            //source is not crypted but host seems to think it should be
            //crypting add 4096 header on each file with a size greater than 4096 bytes
            //when asking size host remove 4096 bytes to answer
            //it do so on my uncrypted source
            //si if size of destination is above 4096 we must add 4096 to size of source when comparing
            if f.len >= 4096 {
                //we look at destination size which is the only good one
                //if more than 4096 then we remove them from destination
                if (self.len + 4096) != f.len {
                    return FicComp::SizeChange(self.len, f.len);
                }
            } else {
                //less than 4096, no crypting so direct compare
                if self.len != f.len {
                    return FicComp::SizeChange(self.len, f.len);
                }
            }
        } else {
            if self.len != f.len {
                return FicComp::SizeChange(self.len, f.len);
            }
        }
        /*if self.name != f.name
        {
            return true;
        }*/
        if self.len != 0 {
            if self.modify != f.modify {
                return FicComp::DateChange(self.modify, f.modify);
            }
        }
        FicComp::Same
    }
}
