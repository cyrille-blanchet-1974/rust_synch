mod explorer;
mod paramcli;

use explorer::*;
use paramcli::*;

use std::path::Path;


pub struct Biexplorer
{
    param: Paramcli,
    src: Dossier,
    dst: Dossier,
}

impl Biexplorer{
    pub fn new()->Biexplorer
    {
        let param = Paramcli::new();
        println!("params: {:?}", param );
        /* 
        cargo run /src:c:\ /dst:"f:\windows XP" /fic:run.cmd /multithread /append /verbose /Crypt /Ignore_Err
        -> params: Paramcli { source: "c:\\", destination: "f:\\windows XP", fic_out: "run.cmd", multithread: true, append: true, verbose: true, crypt: true, ignore_err: true }

        cargo run /src:c:\ /dst:"f:\windows XP" /fic:run.cmd 
        -> params: Paramcli { source: "c:\\", destination: "f:\\windows XP", fic_out: "run.cmd", multithread: false, append: false, verbose: false, crypt: false, ignore_err: false }
        */

        let mut explorer = Explorer::new();
        let src = explorer.run(&Path::new(param.source.as_str()));
        let dst = explorer.run(&Path::new(param.destination.as_str()));

        Biexplorer{
            param: param,
            src: src,
            dst: dst,
        }
    }

    pub fn display_source(&self)
    {
        self.src.display(&Path::new(self.param.source.as_str()));
    }
    pub fn display_destination(&self)
    {
        self.dst.display(&Path::new(self.param.destination.as_str()));
    }
}