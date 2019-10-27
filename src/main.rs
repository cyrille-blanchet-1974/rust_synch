mod explore;
use explore::*;

use std::io;
use std::path::Path;
use std::time::SystemTime;

use structopt::StructOpt;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(StructOpt,Debug)]
#[structopt(name = "synch", about = "utility to synchronize to folders.")]
struct Cli {
    #[structopt(short="src", long="source", parse(from_os_str))]
    src: std::path::PathBuf,

    #[structopt(short="dst", long="destination", parse(from_os_str))]
    dst: std::path::PathBuf,

    #[structopt(short="fic", long="fichier_cmd", parse(from_os_str))]
    fic: std::path::PathBuf,

    #[structopt(short="m", long="multithread")]
    multithread: bool,

    #[structopt(short="a", long="append")]
    append: bool,

    #[structopt(short="v", long="verbose")]
    verbose: bool,

    #[structopt(short="c", long="crypt")]
    crypt: bool,

    #[structopt(short="i", long="ignore_err")]
    ignore_err: bool,
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
    let cli = Cli::from_args();
    println!("{:?}", cli);

    /*
----------------------------------------------------------------------------------------------------------------------------------
PS F:\dev\rust\synch_5> .\target\debug\synch.exe /DDD
error: Found argument '/DDD' which wasn't expected, or isn't valid in this context

USAGE:
    synch.exe [FLAGS] --destination <dst> --fichier_cmd <fic> --source <src>

For more information try --help
----------------------------------------------------------------------------------------------------------------------------------
F:\dev\rust\synch_5> .\target\debug\synch.exe --help
synch 0.1.0
cyrille
utility to synchronize to folders.

USAGE:
    synch.exe [FLAGS] --destination <dst> --fichier_cmd <fic> --source <src>
FLAGS:
    -a, --append
    -c, --crypt
    -h, --help           Prints help information
    -i, --ignore_err
    -m, --multithread
    -V, --version        Prints version information

OPTIONS:
    -d, --destination <dst>
    -f, --fichier_cmd <fic>
    -s, --source <src>
PS F:\dev\rust\synch_5> .\target\debug\synch.exe --src c:\ --dst f:\ --fic batch.cmd -a -c -i -m -v
error: Found argument '--src' which wasn't expected, or isn't valid in this context
        Did you mean --source?

USAGE:
    synch.exe [FLAGS] --destination <dst> --fichier_cmd <fic> --source <src>

For more information try --help
----------------------------------------------------------------------------------------------------------------------------------
F:\dev\rust\synch_5> .\target\debug\synch.exe --source c:\ --destination f:\ --fichier_cmd batch.cmd -a -c -i -m -v
Cli { src: "c:\\", dst: "f:\\", fic: "batch.cmd", multithread: true, append: true, verbose: true, crypt: true, ignore_err: true }    
    */

    
    pause();
    let start = SystemTime::now();
    let root = Path::new("c:\\");
    let mut explore = Explore::new();
    let _data = explore.run(&root);
    let end = SystemTime::now();
    println!("Duration: {:?}", end.duration_since(start).expect("Error computing duration!")  );
    //println!("Total {}/{} dir && {}/{} files",explore.folder_explored_count,explore.folder_forbidden_count,explore.file_explored_count,explore.file_forbidden_count);
    explore.display_count();
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
