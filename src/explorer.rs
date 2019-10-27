pub mod fold;
pub use fold::*;

use std::fs;
use std::path::Path;
use std::time::{SystemTime,Duration};


pub struct Explorer
{
    folder_explored_count: u32,
    folder_forbidden_count: u32,
    file_explored_count: u32,
    file_forbidden_count: u32,   
}

impl Explorer{
    pub fn new()->Explorer
    {
        Explorer{
            folder_explored_count: 0,
            folder_forbidden_count: 0,
            file_explored_count: 0,
            file_forbidden_count: 0,   
        }
    }

    pub fn run(&mut self,dir: &Path) -> Fold
    {
        self.folder_explored_count = 0;
        self.folder_forbidden_count = 0;
        self.file_explored_count = 0;
        self.file_forbidden_count = 0;
        let start = SystemTime::now();
        let mut d = Fold::new_root(dir);
        self.run_int(dir,&mut d);
        let end = SystemTime::now();
        let tps = end.duration_since(start).expect("Error computing duration!");
        self.display_debug(dir, tps);
        d
    }

    //internal function called by run so run could do some 1st call things without testing at each runs
    fn run_int(&mut self,dir: &Path,fold : &mut Fold)
    {
        //count for this level of folder
        if dir.is_dir() {
            let dir_content = fs::read_dir(dir);
            match dir_content 
            {
                Err(e) => {
                    println!("Error {} on {:?}",e,dir);//appears on folder of which i have no access right
                    self.folder_forbidden_count +=1;
                    fold.forbidden=true;
                    }, 
                Ok(content) => {
                    for entry in content {
                        match entry
                        {
                            Err(e) => println!("Error {}",e),
                            Ok(e) => {
                                let path = e.path();
                                if path.is_dir() {
                                    self.folder_explored_count += 1;
                                    let mut sub_fold = Fold::new(&path);
                                    self.run_int(&path,&mut sub_fold);
                                    fold.add_fold(sub_fold);
                                } else {
                                    self.file_explored_count += 1;
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

    fn display_debug(&self,dir: &Path, tps:Duration)
    {
        println!("Folder: {:?}",dir);
        println!("Folder total/forbidden {}/{} ",self.folder_explored_count,self.folder_forbidden_count);
        println!("Files total/forbidden {}/{}",self.file_explored_count,self.file_forbidden_count);
        println!("Duration {:?}",tps);
    }
}
