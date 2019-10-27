use std::fs;
use std::io;
use std::path::{Path,PathBuf};
use std::time::SystemTime;

//stocker dans une structure ou un hashmap
struct Fichier
{
    /*create : SystemTime,
    modify : SystemTime,*/
    path : Box<PathBuf>,
    prop : fs::Metadata,
}

impl Fichier{
    fn new(p : &Path)->Option<Fichier>
    {
        //let dir = *path.path();
        let md = p.metadata();
        match md
        {
            Err(e) =>  None,
            Ok(m) => {
                let pb:PathBuf = (*p).to_path_buf();
                let bp = Box::new( pb );
                Some(Fichier{path: bp, prop:m})
            }
        }
    }
}

struct Dossier
{
    fichiers : Vec<Fichier>,
    dossiers : Vec<Dossier>,
}

impl Dossier{
    fn new()->Dossier
    {
        Dossier{fichiers:Vec::new(),dossiers:Vec::new()}
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

// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path,total_dir: &mut u32,total_fic: &mut u32,doss : &mut Dossier ){
    let mut nb_dir=0;
    let mut nb_fic=0;
    if dir.is_dir() {
        let dir_content = fs::read_dir(dir);
        match dir_content 
        {
            Err(e) => println!("Error {} on {:?}",e,dir),
            Ok(content) => {
                for entry in content {
                    match entry
                    {
                        Err(e) => println!("Error {}",e),
                        Ok(e) => {
                            let path = e.path();
                            if path.is_dir() {
                                nb_dir +=1;
                                let mut sous_doss = Dossier::new();
                                visit_dirs(&path,total_dir,total_fic,&mut sous_doss);
                                doss.add_dossier(sous_doss);
                            } else {
                                nb_fic +=1;
                                let fic = Fichier::new(&path);
                                if fic.is_some()
                                {
                                    doss.add_fichier(fic.unwrap());
                                }
                                /*
                                let md = dir.metadata();
                                match md
                                {
                                    Err(e) =>  println!("Error {}",e),
                                    Ok(m) => {
                                        //date création 
                                        let date_creation:SystemTime;
                                        if let Ok(date_creation) = m.created() {
                                        } else {
                                            println!("Not supported on this platform");
                                        }
                                        let date_modif:SystemTime;
                                        if let Ok(date_modif) = m.modified() {
                                        } else {
                                            println!("Not supported on this platform");
                                        }
                                        let fsize:u64 = m.len();
                                    }
                                } */
                            }
                        }
                    };
                }
            }
        }
    }
    //println!("{:?} contient {} dossiers et {} fichiers",dir,nb_dir,nb_fic);
    *total_dir = *total_dir + nb_dir;
    *total_fic = *total_fic + nb_fic;
    //Ok(())
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
    let mut total_dir = 0;
    let mut total_fic = 0;
    let mut doss = Dossier::new();
    let root = Path::new("c:\\");
    visit_dirs(root,&mut total_dir,&mut total_fic,&mut doss);
    let end = SystemTime::now();
    println!("Duration: {:?}", end.duration_since(start).expect("Error computing duration!")  );
    println!("Total {} dir && {} files",total_dir,total_fic);

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
}
