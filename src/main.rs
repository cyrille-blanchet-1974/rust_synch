use std::fs;
use std::io;
use std::ffi::OsString;
use std::path::{Path,PathBuf};
use std::time::SystemTime;

struct Fichier
{
    modify : SystemTime,
    len : u64,
    name : OsString,
}

impl Fichier{
    fn new(p : &Path)->Option<Fichier>
    {
        let m : SystemTime;
        let l : u64;
        let n : OsString;
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
                Some(Fichier{
                    modify : m,
                    len : l,
                    name : n
                    })
            }
        }
    }
}

struct Dossier
{
    name : OsString,            //folder's name (only it : not the full path!!)
    fichiers : Vec<Fichier>,    //files inside the folder
    dossiers : Vec<Dossier>,    //sub-filders
}

impl Dossier{
    fn new(dir : &Path)->Dossier
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

    fn add_dossier(&mut self, doss : Dossier)
    {
        self.dossiers.push(doss);
    }

    fn add_fichier(&mut self, fic : Fichier)
    {
        self.fichiers.push(fic);
    }
}

struct Explore
{
    folder_explored_count: u32,
    folder_forbidden_count: u32,
    file_explored_count: u32,
    file_forbidden_count: u32,   
}

impl Explore{
    fn new()->Explore
    {
        Explore{
            folder_explored_count: 0,
            folder_forbidden_count: 0,
            file_explored_count: 0,
            file_forbidden_count: 0,   
        }
    }

    fn run(&mut self,dir: &Path) -> Dossier
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
}


fn pause()
{
    //'pause' //press enter
    println!("Pause: press Enter");
    let mut _res = String::new();
    io::stdin().read_line(&mut _res).expect("Failed to read line");
    //let res = _res.trim();
}


fn main() {
    pause();
    let start = SystemTime::now();
    let root = Path::new("c:\\");
    let mut explore = Explore::new();
    let _data = explore.run(&root);
    let end = SystemTime::now();
    println!("Duration: {:?}", end.duration_since(start).expect("Error computing duration!")  );
    println!("Total {}/{} dir && {}/{} files",explore.folder_explored_count,explore.folder_forbidden_count,explore.file_explored_count,explore.file_forbidden_count);
    pause();
    //pour synch_1:
    //premier parcours de C:\ Dureation: 582.7216589s (affichage de chaque dossier avec nb dir et nb fic)
    //second parcours de C:\ (bénéficiant du cache) : 35.7666452s comptage ne fonctionne pas
    //troisième essai ok : 33.9084728s     56 463 dir && 292 194 files
    //4ème (après reboot): 61.6650763s     56 467 dir && 294 318 files
    //      taille mémoire a la première pause : 340Ko, à la seconde pause : 720Ko
    //->4eme = release

    //pour synch_2:
    //1er (après reboot mais avec stockage )Dureation: 79.4433465s Total 56468 dir && 294203 files
    //2eme avec données en cache mais stockage en ram Duration: 41.0292103s   Total 56468 dir && 294204 files
    //          mémoire au lancement    316K   taille mémoire a fin 62 148K
    //3eme  368Ko -> 62 264K Duration: 43.695013s Total 56470 dir && 294322 files
    //4eme  296Ko -> 62 032K Duration: 44.2833532s Total 56470 dir && 294322 files
    //correction de bug (je passait toujours le même chemin a Fichier::new()  
    //5ème  276Ko -> 71 512K Duration: 39.9544583s Total 56472 dir && 294361 files

    //pour syynch_3
    //on ne gardant que nom,  taille et date modif dans Fichier
    //316Ko -> 34 028Ko Duration: 42.6894236s Total 56472 dir && 294360 files
    //320kO -> 38 392kO Duration: 49.4006888s Total 56478/60 dir && 294515/476 files
}
