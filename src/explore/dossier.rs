mod fichier;
pub use fichier::*;

use std::ffi::OsString;
use std::path::Path;

pub struct Dossier
{
    name : OsString,            //folder's name (only it : not the full path!!)
    fichiers : Vec<Fichier>,    //files inside the folder
    dossiers : Vec<Dossier>,    //sub-filders
}

impl Dossier{
    pub fn new(dir : &Path)->Dossier
    {
        let n = 
        match (*dir).file_name()
        {
            Some(n) => n.to_os_string(), // pour un dossier ou un fichier quelconque => son nom
            None => (*dir).as_os_str().to_os_string(),  //pour / ou c:\ ...   on garde tout
        };
        Dossier{
            name : n, //(*dir.file_name().unwrap()).to_os_string(), //will fail on "." and ".."
            fichiers:Vec::new(),
            dossiers:Vec::new()}
    }

    pub fn add_dossier(&mut self, doss : Dossier)
    {
        self.dossiers.push(doss);
    }

    pub fn add_fichier(&mut self, fic : Fichier)
    {
        self.fichiers.push(fic);
    }
}
