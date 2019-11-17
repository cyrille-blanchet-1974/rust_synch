use super::explorer::*;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread::{spawn, JoinHandle};
use std::time::{SystemTime,Duration};

/** 
 * start a thread that reads a MPSC chanel
 * when a source and a destination data is receive we create a tuple and send it to the 2 MPSC chanels to compare threads
 * the data comes from a receiver part of chanel (first arg)
 * the second and third arguments are the sender part of two MPSC chanels that goes to comparison threads
*/
pub fn start_thread_joiner(from_read: Receiver<(Place,Fold)>, to_comp_p: Sender<(Arc<Fold>,Arc<Fold>)>, to_comp_m: Sender<(Arc<Fold>,Arc<Fold>)>) -> JoinHandle<()>
{
    let handle = spawn( move || {
        println!("INFO start joiner");
        //elapse timings (duration of thread)
        let start_elapse = SystemTime::now();
        let mut tps = Duration::new(0, 0);
        let mut src = VecDeque::new();
        let mut dst = VecDeque::new();
        //counts src and dst receive and comparisons sends
        let mut nb_src=0;
        let mut nb_dst=0;
        let mut nb_comp=0;
        //iterate on chanel
        for (typ, data) in from_read{
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
                if to_comp_m.send( (s.clone(),d.clone()) ).is_err()
                {
                    println!("ERROR calling comp_m");
                    return;
                }
                if to_comp_p.send( (s,d) ).is_err()
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
