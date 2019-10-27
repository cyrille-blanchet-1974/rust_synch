mod paramcli;
mod explorer;

use paramcli::*;
use explorer::*;

use std::io;
use std::path::Path;

fn pause()
{
    //'pause' //press enter
    println!("Pause: press Enter");
    let mut _res = String::new();
    io::stdin().read_line(&mut _res).expect("Failed to read line");
    //let res = _res.trim();
}

fn read( d : (&String,&String)) -> (Fold,Fold)
{
    let mut explorer = Explorer::new();
    let src = explorer.run(&Path::new(d.0.as_str()));
    pause();
    let dst = explorer.run(&Path::new(d.1.as_str()));
    (src,dst)
}

fn display( d : &(Fold,Fold)) 
{
    d.0.display();
    d.1.display();
}

fn compare( d : &(Fold,Fold)) 
{
    d.1.gen_remove(&(d.0));
    d.0.gen_copy(&(d.1));
}

fn main() {
    pause();
    let param = Paramcli::new();
    if param.verbose
    {
        println!("params: {:?}", param );
        pause();
    }
    /* 
    cargo run /src:c:\ /dst:"f:\windows XP" /fic:run.cmd /multithread /append /verbose /Crypt /Ignore_Err
    -> params: Paramcli { source: "c:\\", destination: "f:\\windows XP", fic_out: "run.cmd", multithread: true, append: true, verbose: true, crypt: true, ignore_err: true }

    cargo run /src:c:\ /dst:"f:\windows XP" /fic:run.cmd 
    -> params: Paramcli { source: "c:\\", destination: "f:\\windows XP", fic_out: "run.cmd", multithread: false, append: false, verbose: false, crypt: false, ignore_err: false }
    */

    let d = read( (&param.source,&param.destination) );
    pause();
    if param.verbose
    {
        display(&d);
        pause();
    }
    compare(&d);
    pause();
}
