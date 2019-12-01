use std::ffi::OsString;
use std::path::Path;
use std::time::SystemTime;

extern crate filetime;
use filetime::FileTime;

//File properties
pub struct Fic {
    pub forbidden: bool,    //Forbidden folder
    pub modify: SystemTime, //modification date
    pub modify_second: i64, //same but in seconds
    pub len: u64,           //length
    pub name: OsString,     //name
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
    pub fn new(path: &Path) -> Fic {
        let modify: SystemTime; //last modification date of the file
        let len: u64; //length of the file
        let name: OsString; //name of the file
        let modify_second: i64; //modify in seconds
        let mut forbidden = false;
        match (*path).file_name() {
            None => {
                //should only fail if path is a folder and not a file...
                println!("This error should only appear while developping !!! {:?} is a folder and not a file!",path);
                std::process::exit(-2);
            }
            Some(nam) => {
                name = nam.to_os_string();
            }
        }
        //read file infos
        match path.metadata() {
            Err(err) => {
                println!("Error with metadata of {:?} -> {}", path, err); //appears on files of which i have no access right
                forbidden = true;
                len = 0;
                modify = SystemTime::now();
                modify_second = 0;
            }
            Ok(metadata) => {
                len = metadata.len();
                modify = match metadata.modified() {
                    Err(e) => {
                        println!("It seems that your filesystem does not support modification time! It's odd. You should probably not use this prog on this system => {}",e);
                        std::process::exit(-3);
                    }
                    Ok(data) => data,
                };
                let mtime = FileTime::from_last_modification_time(&metadata);
                modify_second = mtime.unix_seconds();
            }
        }

        //we have all we need to produce our File struct
        Fic {
            forbidden,
            modify,
            modify_second,
            len,
            name,
        }
    }

    pub fn comp(&self, other: &Fic, crypt: bool) -> FicComp {
        if crypt {
            //host use crypting
            //source is not crypted but host seems to think it should be
            //crypting add 4096 header on each file with a size greater than 4096 bytes
            //when asking size host remove 4096 bytes to answer
            //it do so on my uncrypted source
            //si if size of destination is above 4096 we must add 4096 to size of source when comparing
            if other.len >= 4096 {
                //we look at destination size which is the only good one
                //if more than 4096 then we remove them from destination
                if (self.len + 4096) != other.len {
                    return FicComp::SizeChange(self.len, other.len);
                }
            } else {
                //less than 4096, no crypting so direct compare
                if self.len != other.len {
                    return FicComp::SizeChange(self.len, other.len);
                }
            }
        } else if self.len != other.len {
            return FicComp::SizeChange(self.len, other.len);
        }
        /*if self.name != f.name
        {
            return true;
        }*/
        if self.len != 0 {
            //the modify data is a very big number
            //precision is 0.0000001 seconds !
            //will compare on second level max
            //modulo 10000000
            /*if self.modify != f.modify {
                return FicComp::DateChange(self.modify, f.modify);
            }*/
            if self.modify_second != other.modify_second {
                return FicComp::DateChange(self.modify, other.modify);
            }
        }
        FicComp::Same
    }
}
