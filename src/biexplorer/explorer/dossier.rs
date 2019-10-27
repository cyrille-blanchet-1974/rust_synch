mod fichier;
pub use fichier::*;

use std::ffi::OsString;
use std::path::Path;
use std::collections::HashMap;

pub struct Dossier
{
    pub name : OsString,            //folder's name (only it : not the full path!!)
    pub fichiers : HashMap<OsString,Fichier>,    //files inside the folder
    pub dossiers : HashMap<OsString,Dossier>,    //sub-filders
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
            name : n, 
            fichiers:HashMap::new(),
            dossiers:HashMap::new()}
    }

    pub fn add_dossier(&mut self, doss : Dossier)
    {
        let n = Path::new(&doss.name);
        let name = (n.file_name().unwrap()).to_os_string();
        self.dossiers.insert(name, doss);
    }

    pub fn add_fichier(&mut self, fic : Fichier)
    {
        let n = Path::new(&fic.name);
        let name = (n.file_name().unwrap()).to_os_string();
        self.fichiers.insert(name, fic);
    }

    pub fn display(&self,racine:&Path)
    {
        let nouvelle_racine = Path::new(racine).join(&self.name);
        println!("{:?}",&nouvelle_racine);
        for (_key, val) in self.fichiers.iter()
        {
            let nouveau_fichier = Path::new(&nouvelle_racine).join(&val.name);
            println!("{:?}",nouveau_fichier);
        }
        for (_key, val) in self.dossiers.iter()
        {
            val.display(&nouvelle_racine);
        }
    }    
}
