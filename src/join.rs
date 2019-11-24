use super::constant::*;
use super::explorer::*;
use super::fold::*;
use super::logger::*;

use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, SystemTime};

/**
 * start a thread that reads a MPSC chanel
 * when a source and a destination data is receive we create a tuple and send it to the 2 MPSC chanels to compare threads
 * the data comes from a receiver part of chanel (first arg)
 * the second and third arguments are the sender part of two MPSC chanels that goes to comparison threads
*/
pub fn start_thread_joiner(
    from_read: Receiver<(Place, Fold)>,
    to_comp_p: Sender<(Arc<Fold>, Arc<Fold>)>,
    to_comp_m: Sender<(Arc<Fold>, Arc<Fold>)>,
    to_logger: Sender<String>,
    verbose: bool,
) -> JoinHandle<()> {
    let handle = spawn(move || {
        let logger = Logger::new(JOINER.to_string(), verbose, to_logger);
        logger.starting();
        //elapse timings (duration of thread)
        let start_elapse = SystemTime::now();
        let mut tps = Duration::new(0, 0);
        let mut src = VecDeque::new();
        let mut dst = VecDeque::new();
        //counts src and dst receive and comparisons sends
        let mut nb_src = 0;
        let mut nb_dst = 0;
        let mut nb_comp = 0;
        //iterate on chanel
        for (typ, data) in from_read {
            logger.verbose(format!("receive {} data", typ.to_string()));
            //real timing
            let start = SystemTime::now();
            //save the data receive in a list regarding of his type
            match typ {
                Place::Src => {
                    src.push_back(data);
                    nb_src += 1;
                }
                Place::Dst => {
                    dst.push_back(data);
                    nb_dst += 1;
                }
            }
            if !src.is_empty() && !dst.is_empty() {
                logger.verbose("got a pair -> sending to comparers".to_string());
                //when we have at least a source and a destination we have our tuple
                let s = Arc::new(src.pop_front().unwrap()); //unwrap should not panic. pop must work as we check emptiness below
                let d = Arc::new(dst.pop_front().unwrap());
                //and send then to the comparison threads
                //note thart we remove the data from the lists because we don't need to keep them after they are sent
                if to_comp_m.send((s.clone(), d.clone())).is_err() {
                    logger.error("calling comp_m".to_string());
                    return;
                }
                if to_comp_p.send((s, d)).is_err() {
                    logger.error("calling comp_p".to_string());
                    return;
                }
                nb_comp += 1;
            }
            tps += logger.timed("finished one operation".to_string(), start);
        }
        logger.dual_timed(
            format!(
                "finished all (src:{} dst:{} cmp:{})",
                nb_src, nb_dst, nb_comp
            ),
            tps,
            start_elapse,
        );
    });
    handle
}
