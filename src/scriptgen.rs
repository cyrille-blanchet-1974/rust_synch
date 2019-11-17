use std::ffi::OsString;
use std::path::PathBuf;
use std::io::{Write,BufWriter};
use std::fs::File; 
use std::sync::mpsc::Receiver;
use std::thread::{spawn, JoinHandle};
use std::time::{SystemTime,Duration};

pub enum Command
{
    Copy(PathBuf, PathBuf),
    CopyRecurs(PathBuf, PathBuf),
    RemoveFile(PathBuf),
    RemoveFold(PathBuf, u32, u32),    
}

impl Command
{
    pub fn to_command(&self)->OsString
    {
        match self
        {
            Command::Copy(src, dst) => gen_copy(src, dst),
            Command::CopyRecurs(src, dst) => gen_copy_rec(src, dst),
            Command::RemoveFile(dst) => gen_del(dst),
            Command::RemoveFold(dst, nbfold, nbfic) => gen_rd(dst, nbfold, nbfic),
        }
    }
}

pub fn gen_copy(src: &PathBuf, dst: &PathBuf)->OsString
{
    let mut res = OsString::new();
    res.push(r###"XCOPY ""###);
    res.push(src);
    res.push(r###"" ""###);
    res.push(dst);
    res.push(r###"" /H /Y /K /R "###);
    // /H   also copy hidden files
    // /Y   No confirmation ask to user
    // /K   copy attributes
    // /R   replace Read only files
    res
}

pub fn gen_copy_rec(src: &PathBuf, dst: &PathBuf)->OsString
{
    let mut res = OsString::new();
    res.push(r###"XCOPY ""###);
    res.push(src);
    res.push("\\*.*");
    res.push(r###"" ""###);    
    res.push(dst);
    res.push(r###"" /E /I /H /Y /K /R "###);
    // /E   copy empty sub folders
    // /I   choose folder as destination if many files in source
    // /H   also copy hidden files
    // /Y   No confirmation ask to user
    // /K   copy attributes
    // /R   replace Read only files
    res
}

pub fn gen_del(dst: &PathBuf)->OsString
{
    let mut res = OsString::new();
    res.push(r###"DEL ""###);
    res.push(dst);
    res.push(r###"" /F /A "###);
    //   /F   Force delete of read only
    //   /A   delete whatever attributes 
    res
}

pub fn gen_rd(dst: &PathBuf, nbfic: &u32, nbfold: &u32)->OsString
{
    let mut res = OsString::new();
    if *nbfold > 10 || *nbfic > 100
    {
        let s = format!("Echo {:?} Contains {} folders and {}  files.\n",dst,nbfold,nbfic);
        res.push(s);
        res.push("Echo Please confirm deletation\n");
        res.push("Echo Y to Delete\n");
        res.push("Echo N to keep\n");
        res.push("choice /C YN\n");
        res.push("if '%ERRORLEVE%'=='1' ");   
    }
    res.push(r###"RD /S /Q ""###);
    res.push(dst);
    res.push(r###"""###);
    //   /S   recursive
    //   /Q   No confirmation ask to user
    res
}

/**
* start output thread
* we open/create the destination file
* for each command receive from the chanel we write in output
* 
*/
pub fn start_thread_writer(from_comp: Receiver<Command>, output: PathBuf) -> JoinHandle<()>
{
   let handle = spawn( move || {
       let start_elapse = SystemTime::now();
       let mut tps = Duration::new(0, 0);
       println!("INFO start writer");
       let mut nb_ecr = 0;
       let writer = 
       match File::create(output)
       {
               Err(e) =>{
                   println!("Erreur écriture fichier {:?}", e);
                   return;
               },
               Ok(fichier) =>
               {             
                   fichier
               }
       };
       let mut buffer_writer = BufWriter::new(writer);
       match buffer_writer.write_all("@echo off\n".as_bytes()) 
       {
           Err(e) =>{
               println!("Erreur écriture fichier {:?}", e);
               return;
           },
           Ok(_) =>
           {              
               nb_ecr +=1;        
           }
       } 
       match buffer_writer.write_all("chcp 65001\n".as_bytes()) //utf8 codepage
       {
           Err(e) =>{
               println!("Erreur écriture fichier {:?}", e);
               return;
           },
           Ok(_) =>
           {              
               nb_ecr +=1;        
           }
       } 
       for data in from_comp{
           let start = SystemTime::now();
           match buffer_writer.write_all(data.to_command().to_str().unwrap().as_bytes()) 
           {
               Err(e) =>{
                   println!("Erreur écriture fichier {:?}", e);
                   return;
               },
               Ok(_) =>
               {                      
                   nb_ecr +=1;
               }
           } 
           match buffer_writer.write_all("\n".as_bytes()) 
           {
               Err(e) =>{
                   println!("Erreur écriture fichier {:?}", e);
                   return;
               },
               Ok(_) =>
               {              
                   nb_ecr +=1;        
               }
           } 
           let end = SystemTime::now();
           tps += end.duration_since(start).expect("ERROR computing duration!");
       }
       let end_elapse = SystemTime::now();
       let tps_elapse = end_elapse.duration_since(start_elapse).expect("ERROR computing duration!");
       println!("INFO {} lignes writes in {:?}/{:?}", nb_ecr, tps,tps_elapse);

   });
   handle
}
