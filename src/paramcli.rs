pub use super::readconf::*;

use std::env;

#[derive(Debug)]
pub struct Paramcli
{
    pub source: Vec<String>,
    pub destination: Vec<String>,
    pub fic_out : String,
    pub verbose : bool,
    pub crypt : bool,
    pub ignore_err : bool,
    pub config: String,
}

impl Paramcli{
    pub fn new()->Paramcli
    {
        let mut fic= String::new();
        let mut ver = false;
        let mut cry = false;
        let mut ign = false;
        let mut src = Vec::new();
        let mut dst = Vec::new();
        let mut conf= String::new();

        let args: Vec<String> = env::args().skip(1).collect();
        for arg in args {
            println!("{}",arg);
            if arg == "/?" ||  arg == "-?" || arg.to_lowercase() == "/help" || arg.to_lowercase() == "-help"
            {
                help();
            }
            if arg.to_lowercase().starts_with("/src:")
            {
                src.push(get_param(arg));
                continue;
            }
            if arg.to_lowercase().starts_with("/dst:")
            {
                dst.push(get_param(arg));
                continue;
            }
            if arg.to_lowercase().starts_with("/conf:")
            {
                conf = get_param(arg);
                continue;
            }
            if arg.to_lowercase().starts_with("/fic:")
            {
                fic = get_param(arg);
                continue;
            }
            if arg.to_lowercase() == "/verbose"
            {
                ver = true;
                continue;
            }
            if arg.to_lowercase() == "/crypt"
            {
                cry = true;
                continue;
            }
            if arg.to_lowercase() == "/ignore_err"
            {
                ign = true;
                continue;
            }
        }
        if !conf.is_empty()
        {
            let readconf = Readconf::new(&conf);
            src = readconf.source;
            dst = readconf.destination;
            //TODO: when data incorrect display a more friendly message
            assert_eq!(src.len(),dst.len());
            assert_ne!(src.len(),0);
            for i in 1..src.len() {
                assert_ne!(src.get(i),dst.get(i));
            }
        }
        Paramcli{
            source: src,
            destination: dst,
            fic_out : fic,
            verbose : ver,
            crypt : cry,
            ignore_err : ign,
            config : conf
        }
    }
}

fn get_param(arg : String) -> String
{
    let mut res = String::new();
    for part in arg.split(":").skip(1)
    {
        if !res.is_empty()
        {
            res.push_str(":");
        }
        res.push_str(part);
    }
    if arg.ends_with(":")
    {
        res.push_str(":");
    }
    res
}

fn help()
{
	println!("{}","Syntaxe: synch /src:dossier_source /des:dossier_cible /fic:fichier_sortie.bat [/append] [/multithread] [/verbose] [/crypt] [/ignore_err]");
    println!("{}","------------------------------------");
    println!("{}","dossier_source: Dossier maître");
    println!("{}","dossier_cible: Dossier esclave (deviendra un clone de source)");
    println!("{}","fichier_sortie.bat: fichier bat qui recevra les commandes pour cloner source en cible");
    println!("{}","/multithread: Option pour mode multithread");
	println!("{}","/append: Indique si on ajoue le résulat au fichier de sortie (défaut = écraser)");
	println!("{}","/verbose: affiche a l'écran les information indiquant les différences sources/cible");
	println!("{}","/crypt: Si la destination fait exactement 4096 octet de moins concidérer comme identique");
	println!("{}","/ignore_err: ne pas s'arrêter en cas d'erreur");
    println!("{}","---------------------------------------------------------------------------");
    std::process::exit(0);
}