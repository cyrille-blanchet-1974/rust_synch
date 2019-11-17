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
            Some(n) => n.to_os_string(), // pour un Fold ou un fic quelconque => son nom
            None => (*dir).as_os_str().to_os_string(), //pour / ou c:\ ...   on garde tout
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
        let name = (n.file_name().unwrap()).to_os_string();
        self.folder_count += 1;
        let d = fold.get_counts();
        self.folder_count += d.0;
        self.file_count += d.1;
        self.folds.insert(to_lower(&name), fold);
    }

    pub fn add_fic(&mut self, fic: Fic) {
        let n = Path::new(&fic.name);
        let name = (n.file_name().unwrap()).to_os_string();
        self.file_count += 1;
        self.fics.insert(to_lower(&name), fic);
    }

    pub fn get_counts(&self) -> (u32, u32) {
        (self.folder_count, self.file_count)
    }
}

pub fn to_lower(name: &OsString) -> OsString {
    let res: OsString;
    let clone = OsString::from(name);
    match clone.into_string() {
        Ok(a) => {
            let a = a.to_lowercase();
            res = OsString::from(a);
        }
        Err(a) => {
            res = a;
        }
    }
    return res;
}
