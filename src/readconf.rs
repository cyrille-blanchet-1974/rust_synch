use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

#[derive(Debug)]
pub struct Readconf
{
    pub source : Vec<String>,
    pub destination: Vec<String>,
}

impl Readconf{
    pub fn new(ficconf:&String)->Readconf
    {
        let mut src= Vec::new();
        let mut dst= Vec::new();

        let input = File::open(&ficconf);
        match input
        {
            Err(e) =>{
                println!("Error reading conf file {} => {}",&ficconf,e);
            },
            Ok(f) =>{
                let buffered = BufReader::new(f);
                for line in buffered.lines() {
                    match line
                    {
                        Ok(l)=>{
                            let res = get_val(&l);
                            if l.to_lowercase().starts_with("source=")
                            {
                                src.push(res);
                            }
                            else
                            {
                                if l.to_lowercase().starts_with("destination=")
                                {
                                    dst.push(res);
                                }
                            }
                        },
                        Err(_)=>{
                        }
                    }
                }
            }
        }

        Readconf{
            source: src,
            destination: dst,
        }
    }
}

fn get_val(arg : &String) -> String
{
    let mut res = String::new();
    for part in arg.split("=").skip(1)
    {
        if !res.is_empty()
        {
            res.push_str("=");
        }
        res.push_str(part);
    }
    res
}