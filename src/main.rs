use std::io;
use std::fs;
use std::path::Path;
use std::time::SystemTime;


// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path,total_dir: &mut u32,total_fic: &mut u32 ){
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
                                visit_dirs(&path,total_dir,total_fic);
                            } else {
                                nb_fic +=1;
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
    let root = Path::new("c:\\");
    visit_dirs(root,&mut total_dir,&mut total_fic);
    let end = SystemTime::now();
    println!("Dureation: {:?}", end.duration_since(start).expect("Error computing duration!")  );
    println!("Total {} dir && {} files",total_dir,total_fic);

    pause();

    //pour synch_1:
    //premier parcours de C:\ Dureation: 582.7216589s (affichage de chaque dossier avec nb dir et nb fic)
    //second parcours de C:\ (bénéficiant du cache) : 35.7666452s comptage ne fonctionne pas
    //troisième essai ok : 33.9084728s     56 463 dir && 292 194 files
    //4ème (après reboot): 61.6650763s     56 467 dir && 294 318 files
    //      taille mémoire a la première pause : 340Ko, à la seconde pause : 720Ko
    //->4eme = release
}
