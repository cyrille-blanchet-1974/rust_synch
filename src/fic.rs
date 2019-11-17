use std::time::SystemTime;
use std::ffi::OsString;
use std::path::Path;

pub struct Fic
{
    pub modify: SystemTime,
    pub len: u64,
    pub name: OsString,
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
    pub fn new(p: &Path)->Option<Fic>
    {
        let m: SystemTime;
        let l: u64;
        let n: OsString;
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
                    modify: m,
                    len: l,
                    name: n
                    })
            }
        }
    }

    pub fn comp(&self, f: &Fic, crypt: bool)->FicComp
    {
        if crypt
        {
             //host use crypting
             //source is not crypted but host seems to think it should be
             //crypting add 4096 header on each file with a size greater than 4096 bytes 
             //when asking size host remove 4096 bytes to answer
             //it do so on my uncrypted source
             //si if size of destination is above 4096 we must add 4096 to size of source when comparing
            if f.len >= 4096
            { //we look at destination size which is the only good one 
		      //if more than 4096 then we remove them from destination
              if (self.len+4096) != f.len
              {
                return FicComp::SizeChange(self.len, f.len);
              } 
            }else
            {
		        //less than 4096, no crypting so direct compare
                 if self.len!=f.len 
                 {
                    return FicComp::SizeChange(self.len, f.len);
                 }
        	}
        }
        else
        {
            if self.len != f.len
            {
                return FicComp::SizeChange(self.len, f.len);
            }
        }     
        /*if self.name != f.name 
        {
            return true;
        }*/
        if self.len != 0 
        {
            if self.modify != f.modify
            {
                return FicComp::DateChange(self.modify, f.modify);
            }
        }
        FicComp::Same
    }
}



