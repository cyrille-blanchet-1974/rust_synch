use std::path::PathBuf;
use std::io::{Write,BufWriter};
use std::fs::{File,OpenOptions}; 

//trait implementation for write 
pub trait Writing {
    fn write(&mut self, data: String);
    fn get_nb_ecr(&self)->i32;
}

pub struct WriterDisk
{
    nb_ecr : i32,
    writer: BufWriter<File>,
}

// write on disk
impl WriterDisk{
    pub fn new(output: PathBuf, create:  bool ) -> Option<WriterDisk> {
        let writer;
        if create 
        { 
            writer = match File::create(output)//append ?
            {
                    Err(e) =>{
                        println!("Erreur création fichier {:?}", e);
                        return None;
                    },
                    Ok(fichier) =>
                    {             
                        fichier
                    }
            };
        }
        else
        { 
            
            writer = match OpenOptions::new().append(true).open(output)
            {
                    Err(e) =>{
                        println!("Erreur ouverture fichier {:?}", e);
                        return None;
                    },
                    Ok(fichier) =>
                    {             
                        fichier
                    }
            };
        }
        Some(WriterDisk {
            nb_ecr : 0,
            writer: BufWriter::new(writer),
        })
    }
}

impl Writing for WriterDisk {
    //how to write a log to disk
    fn write(&mut self, data: String) {
        match self.writer.write_all(data.as_bytes()) 
        {
            Err(e) =>{
                println!("Erreur écriture fichier {:?}", e);
             return;
            },
            Ok(_) =>
            {                      
                self.nb_ecr +=1;
            }
        } 
        match self.writer.write_all("\n".as_bytes()) 
        {
            Err(e) =>{
                println!("Erreur écriture fichier {:?}", e);
             return;
            },
            Ok(_) =>
            {              
                self.nb_ecr +=1;        
            }
        } 
    }
    //how many logs writtens
    fn get_nb_ecr(&self)->i32
    {
        self.nb_ecr
    }
}

//write to console
pub struct WriterConsole
{
    pub nb_ecr : i32,
}

impl WriterConsole{
    pub fn new() -> WriterConsole {
        WriterConsole {
            nb_ecr : 0,
        }
    }
}

impl Writing for WriterConsole {
    //how to write to console
    fn write(&mut self, data: String) {
        println!("{}",data);
        self.nb_ecr +=1;
    }
    //how many logs writtens
    fn get_nb_ecr(&self)->i32
    {
        self.nb_ecr
    }
}
