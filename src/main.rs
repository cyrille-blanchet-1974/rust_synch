mod paramcli;
mod explorer;

use paramcli::*;
use explorer::*;

use std::io::{self,Write};
use std::collections::VecDeque;
use std::path::{Path,PathBuf};
use std::ffi::OsString;
use std::fs::File; 
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::{spawn, JoinHandle};
use std::time::{SystemTime,Duration};

enum Place
{
    Src,
    Dst,
}

impl Place
{
    pub fn to_string(&self)->String
    {
        match self{
            Place::Src => "source".to_string(),
            Place::Dst => "destination".to_string(),
        }
    }
    pub fn clone(&self)->Place
    {
        match self{
            Place::Src => Place::Src,
            Place::Dst => Place::Dst,
        }
    }
}

fn pause()
{
    println!("Pause: press Enter");
    let mut _res = String::new();
    io::stdin().read_line(&mut _res).expect("Failed to read line");
}

/**
 * Note: in the following MPSC chanel means
 * multi-producer single-consumer chanel
 * multiple threads write to the chanel and a unique thread read it
 * 
 * here we use 6 threads (plus the main one) and 4 chanels
 * 2 threads that reads sources and destination and write to the first chanel
 * 1 thread ta read the fist chanel and join sources/destinations to send then
 * to comparisons threads via 2 channels
 * 1 thread to find what is new or modify in source
 * 1 thread to find what no longer exists in source so it must be deleted from destination
 * and 1 las thread that write the commands in a output file
 * see behind a schematic of the threads
 */

/**schamtic: text are threads   and  lines are chanels
 *      read_src         read_dst
 *              \         /
 *               \       /           
 *                \     / 
 *                 \   / 
 *                  \ /
 *                 join
 *                  /\
 *                 /  \
 *                /    \
 *               /      \
 *      find_new        find_to_remove
 *              \         /
 *               \       /           
 *                \     / 
 *                 \   / 
 *                  \ /
 *               write output
 * 
 */

fn start_read_thread(what : Place,sender : Sender<(Place,Fold)>, data : Vec<PathBuf>) -> JoinHandle<()>
{
    let handle = spawn( move || {
        //timings: elapse count all and the other counts only acting time
        let start_elapse = SystemTime::now();
        let mut tps = Duration::new(0,0);
        //number of item receive from chanel
        let mut nb = 0;
        println!("INFO start reading {} folders in {}s",&data.len(),what.to_string());
        let mut explorer = Explorer::new();
        //iterate on sources
        for d in data{
            //read time only
            let start = SystemTime::now();
            let src = explorer.run(&Path::new(d.to_str().unwrap()));
            let end = SystemTime::now();
            tps += end.duration_since(start).expect("ERROR computing duration!");
            //send data to join thread thru MPSC chanel 
            if sender.send((what.clone(),src)).is_err()
            {
                println!("ERROR in start_read_{}",what.to_string());
                return;
            }
            nb +=1;
        }
        let end_elapse = SystemTime::now();
        let tps_elapse = end_elapse.duration_since(start_elapse).expect("ERROR computing duration!");
        println!("INFO {} {} folders read in {:?}/{:?}", what.to_string(),nb, tps,tps_elapse);
    });
    handle
}

/** 
 * start a thread that reads sources and send them in a MPSC chanel
 * the sender part of the chanel is the first argument and receive a tuple containing the type (source or destination)
 * and the full contain of one source (folders and files)
 * the second parameter is a list of source to read
*/
fn start_read_src(sender : Sender<(Place,Fold)>, data : Vec<PathBuf>) -> JoinHandle<()>
{
    start_read_thread(Place::Src,sender,data)
}

/** 
 * start a thread that reads destinations and send them in a MPSC chanel
 * the sender part of the chanel is the first argument and receive a tuple containing the type (source or destination)
 * and the full contain of one destination (folders and files)
 * the second parameter is a list of destinations to read
*/
fn start_read_dst(sender : Sender<(Place,Fold)>, data : Vec<PathBuf>) -> JoinHandle<()>
{
    start_read_thread(Place::Dst,sender,data)
}

/** 
 * start a thread that reads a MPSC chanel
 * when a source and a destination data is receive we create a tuple and send it to the 2 MPSC chanels to compare threads
 * the data comes from a receiver part of chanel (first arg)
 * the second and third arguments are the sender part of two MPSC chanels that goes to comparison threads
*/
fn start_joiner(receiver : Receiver<(Place,Fold)>,sender_comp_p : Sender<(Arc<Fold>,Arc<Fold>)>,sender_comp_m : Sender<(Arc<Fold>,Arc<Fold>)>) -> JoinHandle<()>
{
    let handle = spawn( move || {
        println!("INFO start joiner");
        //elapse timings (duration of thread)
        let start_elapse = SystemTime::now();
        let mut tps = Duration::new(0,0);
        let mut src = VecDeque::new();
        let mut dst = VecDeque::new();
        //counts src and dst receive and comparisons sends
        let mut nb_src=0;
        let mut nb_dst=0;
        let mut nb_comp=0;
        //iterate on chanel
        for (typ,data) in receiver{
            //real timing
            let start = SystemTime::now();
            //save the data receive in a list regarding of his type
            match typ{
                Place::Src => {
                    src.push_back(data);   
                    nb_src +=1;
                },
                Place::Dst => {
                    dst.push_back(data);
                    nb_dst +=1;
                },
            }
            if !src.is_empty() && !dst.is_empty()
            {
                //when we have at least a source and a destination we have our tuple
                let s = Arc::new(src.pop_front().unwrap());
                let d = Arc::new(dst.pop_front().unwrap());
                //and send then to the comparison threads 
                //note thart we remove the data from the lists because we don't need to keep them after they are sent
                if sender_comp_m.send( (s.clone(),d.clone()) ).is_err()
                {
                    println!("ERROR calling comp_m");
                    return;
                }
                if sender_comp_p.send( (s,d) ).is_err()
                {
                    println!("ERROR calling comp_p");
                    return;
                }
                nb_comp +=1;
            }
            let end = SystemTime::now();
            tps += end.duration_since(start).expect("ERROR computing duration!");
        }
        let end_elapse = SystemTime::now();
        let tps_elapse = end_elapse.duration_since(start_elapse).expect("ERROR computing duration!");
        println!("INFO join ends ({} src/ {} dst/ {} comp in {:?}/{:?}", nb_src, nb_dst, nb_comp, tps,tps_elapse);
    });
    handle
}

/**
 * start a first comparison thread
 * we receive data (2 folders) from a chanel 
 * the comparison creates copies commands (for data in source only or in both but with a diff)
 * commands are sent into a output chanel
 * these chanel goes to a thread who is in charge of writing them to outputfile
 */
fn start_comp_p(receiver : Receiver<(Arc<Fold>,Arc<Fold>)>,sender : Sender<OsString>) -> JoinHandle<()>
{
    let handle = spawn( move || {
        let start_elapse = SystemTime::now();
        let mut tps = Duration::new(0,0);
        println!("INFO start comp_p");
        let mut nb_comp=0;
        for (s,d) in receiver{
            let start = SystemTime::now();
            s.gen_copy(&d,&sender);
            nb_comp +=1;
            let end = SystemTime::now();
            tps += end.duration_since(start).expect("ERROR computing duration!");
        }
        let end_elapse = SystemTime::now();
        let tps_elapse = end_elapse.duration_since(start_elapse).expect("ERROR computing duration!");
        println!("INFO {} comp_p in {:?}/{:?}", nb_comp, tps,tps_elapse);
    });
    handle
}

/**
 * quite the same as previous thread
 * but generate remove commands for data in destination only
 */
fn start_comp_m(receiver : Receiver<(Arc<Fold>,Arc<Fold>)>,sender : Sender<OsString>) -> JoinHandle<()>
{
    let handle = spawn( move || {
        let start_elapse = SystemTime::now();
        let mut tps = Duration::new(0,0);
        println!("INFO start comp_m");
        let mut nb_comp=0;
        for (s,d) in receiver{
            let start = SystemTime::now();
            d.gen_remove(&s,&sender);
            nb_comp +=1;
            let end = SystemTime::now();
            tps += end.duration_since(start).expect("ERROR computing duration!");
        }
        let end_elapse = SystemTime::now();
        let tps_elapse = end_elapse.duration_since(start_elapse).expect("ERROR computing duration!");
        println!("INFO {} comp_m in {:?}/{:?}", nb_comp, tps,tps_elapse);
    });
    handle
}

/**
 * start output thread
 * we open/create the destination file
 * for each command receive from the chanel we write in output
 * 
 */
fn start_writer(receiver : Receiver<OsString>,output : PathBuf) -> JoinHandle<()>
{
    let handle = spawn( move || {
        let start_elapse = SystemTime::now();
        let mut tps = Duration::new(0,0);
        println!("INFO start writer");
        let mut nb_ecr = 0;
        let mut writer = 
        match File::create(output)
        {
                Err(e) =>{
                    println!("Erreur écriture fichier {:?}",e);
                    return;
                },
                Ok(fichier) =>
                {             
                    fichier
                }
        };
        match writer.write_all("@echo off\n".as_bytes()) 
        {
            Err(e) =>{
                println!("Erreur écriture fichier {:?}",e);
                return;
            },
            Ok(_) =>
            {              
                nb_ecr +=1;        
            }
        } 
        //todo: use BufWrite
        for data in receiver{
            let start = SystemTime::now();
            match writer.write_all(data.to_str().unwrap().as_bytes()) 
            {
                Err(e) =>{
                    println!("Erreur écriture fichier {:?}",e);
                    return;
                },
                Ok(_) =>
                {                      
                    nb_ecr +=1;
                }
            } 
            match writer.write_all("\n".as_bytes()) 
            {
                Err(e) =>{
                    println!("Erreur écriture fichier {:?}",e);
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

/**
 * entry point of app
 */
fn main() {
    //pause();
    //todo: put param in static so we can access global props everywhere
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
    //list of sources and destinations
    let mut src = Vec::new();
    let mut dst = Vec::new();
    //local
    /*
    src.push(Path::new("c:\\").to_path_buf());
    src.push(Path::new("d:\\").to_path_buf());

    dst.push(Path::new("F:\\").to_path_buf());
    dst.push(Path::new("F:\\").to_path_buf());
    */
    //pc adp
    /*
    src.push(Path::new("C:\\dev").to_path_buf());
    dst.push(Path::new("D:\\pc00916566\\dev").to_path_buf());
    src.push(Path::new("C:\\users\\cyblanc").to_path_buf());
    dst.push(Path::new("D:\\pc00916566\\Users\\cyblanc").to_path_buf());
    src.push(Path::new("C:\\Users\\cyblanc\\Desktop").to_path_buf());
    dst.push(Path::new("D:\\pc00916566\\Bureau").to_path_buf());
    src.push(Path::new("C:\\cvs2git").to_path_buf());
    dst.push(Path::new("D:\\pc00916566\\cvs2git").to_path_buf());
    */
    //test 
    /*
    src.push(Path::new("F:\\dev\\rust\\test_data_synch\\src").to_path_buf());
    dst.push(Path::new("F:\\dev\\rust\\test_data_synch\\dst").to_path_buf());
    */
    //test usb
    /*
    src.push(Path::new("C:\\").to_path_buf());
    dst.push(Path::new("G:\\").to_path_buf());
    */

    //MPSC chanels
      //read threads to join thread
    let (sender_read_to_join, receiver_read_to_join) = channel();
      //join thread to comp plus thread
    let (sender_comp_m, receiver_comp_m) = channel();
      //join thread to comp minus thread    
    let (sender_comp_p, receiver_comp_p) = channel();
      //comp threads to write output
    let (sender_writer, receiver_writer) = channel();

    //local 
    /*
    let hwriter = start_writer(receiver_writer,Path::new("F:\\sortie.cmd").to_path_buf());
    */
    //pc adp
    /* 
    let hwriter = start_writer(receiver_writer,Path::new("sortie.cmd").to_path_buf());
    */
    //test
    /*
    let hwriter = start_writer(receiver_writer,Path::new("F:\\dev\\rust\\test_data_synch\\sortie.cmd").to_path_buf());
    */
    //test usb
    /*
    let hwriter = start_writer(receiver_writer,Path::new("F:\\sortie.cmd").to_path_buf());
    */
    //start writer thread
    let hwriter = start_writer(receiver_writer,Path::new(&param.fic_out).to_path_buf());
    //get data for readers
    //TODO: where param in static move this in start_read_xxx
    for s in param.source {
        src.push(Path::new(&s).to_path_buf());    
    }
    for d in param.destination {
        dst.push(Path::new(&d).to_path_buf());    
    }
    // start compare threads
    let hcompp = start_comp_p(receiver_comp_p, sender_writer.clone());
    let hcompm = start_comp_m(receiver_comp_m, sender_writer);
    //start join thread
    let hjoin = start_joiner(receiver_read_to_join,sender_comp_m,sender_comp_p);
    //start read threads
    let hreadsrc = start_read_src(sender_read_to_join.clone(),src);
    let hreaddst = start_read_dst(sender_read_to_join,dst); 

    //wait for threads to stop
    hreadsrc.join().unwrap();
    hreaddst.join().unwrap();
    hjoin.join().unwrap();
    hcompp.join().unwrap();
    hcompm.join().unwrap();
    hwriter.join().unwrap();

    //pause();
}
