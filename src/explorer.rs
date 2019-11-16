pub use super::fold::*;
pub use super::paramcli::*;

use std::fs;
use std::path::Path;
use std::time::SystemTime;


pub struct Explorer
{
    verbose : bool,
    ignore_err : bool,
    folder_forbidden_count: u32,
    file_forbidden_count: u32,   
}

impl Explorer{
    pub fn new(o : &Options)->Explorer
    {
        Explorer{
            verbose: o.verbose,
            ignore_err : o.ignore_err,
            folder_forbidden_count: 0,
            file_forbidden_count: 0,   
        }
    }

    pub fn run(&mut self,dir: &Path) -> Fold
    {
        if self.verbose
        {
            println!("TRACE Reading Folder: {:?} ",dir);
        }
        self.folder_forbidden_count = 0;
        self.file_forbidden_count = 0;
        let start = SystemTime::now();
        let mut d = Fold::new_root(dir);
        self.run_int(dir,&mut d);
        if self.verbose
        {
            let end = SystemTime::now();
            let tps = end.duration_since(start).expect("ERROR computing duration!");
            let c= d.get_counts();
            println!("TRACE Folder: {:?} Folder total/forbidden {}/{} Files total/forbidden {}/{} Duration {:?}",dir,c.0,self.folder_forbidden_count,c.1,self.file_forbidden_count,tps);
        }
        d
    }

    //internal function called by run so run could do some 1st call things without testing at each runs
    fn run_int(&mut self,dir: &Path,fold : &mut Fold)
    {
        if dir.is_dir() {
            let dir_content = fs::read_dir(dir);
            match dir_content 
            {
                Err(e) => {
                    println!("ERROR {} on {:?}",e,dir);//appears on folder of which i have no access right
                    self.folder_forbidden_count +=1;
                    fold.forbidden=true;
                    }, 
                Ok(content) => {
                    for entry in content {
                        match entry
                        {
                            Err(e) => {
                                        println!("ERROR {}",e);
                                        if !self.ignore_err
                                        {
                                            std::process::exit(0);
                                        }                        
                                     },
                            Ok(e) => {
                                let path = e.path();
                                if path.is_dir() {
                                    let mut sub_fold = Fold::new(&path);
                                    self.run_int(&path,&mut sub_fold);
                                    fold.add_fold(sub_fold);
                                } else {
                                    let fic = Fic::new(&path);
                                    if fic.is_some()
                                    {
                                        fold.add_fic(fic.unwrap());
                                    }
                                    else
                                    {
                                        self.file_forbidden_count +=1;
                                    }
                                }
                            }
                        };
                    }
                }
            }
        }
    }
}
