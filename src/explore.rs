mod dossier;
pub use dossier::*;

use std::fs;
use std::path::Path;


pub struct Explore
{
    folder_explored_count: u32,
    folder_forbidden_count: u32,
    file_explored_count: u32,
    file_forbidden_count: u32,   
}

impl Explore{
    pub fn new()->Explore
    {
        Explore{
            folder_explored_count: 0,
            folder_forbidden_count: 0,
            file_explored_count: 0,
            file_forbidden_count: 0,   
        }
    }

    pub fn run(&mut self,dir: &Path) -> Dossier
    {
        let mut d = Dossier::new(dir);
        self.run_int(dir,&mut d);
        d
    }

    //internal function called by run so run could do some 1st call things without testing at each runs
    fn run_int(&mut self,dir: &Path,doss : &mut Dossier)
    {
        //count for this level of folder
        if dir.is_dir() {
            let dir_content = fs::read_dir(dir);
            match dir_content 
            {
                Err(e) => {
                    println!("Error {} on {:?}",e,dir);//appears on folder of which i have no access right
                    self.folder_forbidden_count +=1;
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
                                    let mut sous_doss = Dossier::new(&path);
                                    self.run_int(&path,&mut sous_doss);
                                    doss.add_dossier(sous_doss);
                                } else {
                                    self.file_explored_count += 1;
                                    let fic = Fichier::new(&path);
                                    if fic.is_some()
                                    {
                                        doss.add_fichier(fic.unwrap());
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

    pub fn display_count(&self)
    {
        println!("Total {}/{} dir && {}/{} files",self.folder_explored_count,self.folder_forbidden_count,self.file_explored_count,self.file_forbidden_count);
    }
}
