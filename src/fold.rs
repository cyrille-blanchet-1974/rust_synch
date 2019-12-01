pub use super::fic::*;

use std::collections::HashMap;
use std::ffi::OsString;
use std::path::Path;

pub struct Fold {
    pub name: OsString,                 //folder's name (only it: not the full path!!)
    pub fics: HashMap<OsString, Fic>,   //files inside the folder
    pub folds: HashMap<OsString, Fold>, //sub-filders
    pub forbidden: bool,                //Forbidden folder
    pub folder_count: u32,              //number of subfolder (recursively)
    pub file_count: u32,                //number of files (recursively)
}

impl Fold {
    pub fn new_root(dir: &Path) -> Fold {
        Fold {
            name: (*dir).as_os_str().to_os_string(),
            fics: HashMap::new(),
            folds: HashMap::new(),
            forbidden: false,
            folder_count: 0,
            file_count: 0,
        }
    }

    pub fn new(dir: &Path) -> Fold {
        let n = match (*dir).file_name() {
            Some(n) => n.to_os_string(), // for a File or a folder we only keep his name
            None => (*dir).as_os_str().to_os_string(), //for root ( / or c:\ ...) we keep the full path
        };
        Fold {
            name: n,
            fics: HashMap::new(),
            folds: HashMap::new(),
            forbidden: false,
            folder_count: 0,
            file_count: 0,
        }
    }

    pub fn add_fold(&mut self, fold: Fold) {
        let n = Path::new(&fold.name);
        let name = match n.file_name() {
            None => {
                //should only fail if path is a root
                println!("This error should only appear while developping !!! {:?} is a root not a subfolder!",n);
                std::process::exit(-2);
            }
            Some(nam) => nam.to_os_string(),
        };
        self.folder_count += 1; //add the folder to  the count
        let d = fold.get_counts();
        self.folder_count += d.0; //add all subfolder to  the count
        self.file_count += d.1; //add all included files ro the count
        self.folds.insert(to_lower(&name), fold);
    }

    pub fn add_fic(&mut self, fic: Fic) {
        let n = Path::new(&fic.name);
        let name = match n.file_name() {
            None => {
                //should only fail if path is a folder and not a file...
                println!("This error should only appear while developping !!! {:?} is a folder and not a file!",n);
                std::process::exit(-2);
            }
            Some(nam) => nam.to_os_string(),
        };
        self.file_count += 1;
        self.fics.insert(to_lower(&name), fic);
    }

    pub fn get_counts(&self) -> (u32, u32) {
        (self.folder_count, self.file_count)
    }
}

pub fn to_lower(name: &OsString) -> OsString {
    let clone = OsString::from(name);
    match clone.into_string() {
        Ok(a) => {
            let a = a.to_lowercase();
            OsString::from(a)
        }
        Err(a) => {
            println!("Error  appears while converting {:?} to uppercase. File/folder may contain non unicode characters!",&name);
            //original osstring return if error
            a
        }
    }
}
