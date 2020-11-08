mod comparer;
mod constant;
mod explorer;
mod fic;
mod fold;
mod join;
mod logger;
mod paramcli;
mod progression;
mod readconf;
mod scriptgen;
mod writer;

use comparer::*;
use constant::*;
use explorer::*;
use join::*;
use logger::*;
use paramcli::*;
use progression::*;
use scriptgen::*;

use std::path::Path;
use std::sync::mpsc::channel;

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

/**
 * entry point of app
 */
fn main() {
    println!("synch 1.0 (2019)");
    let param = Paramcli::new();
    if param.verbose {
        println!("params: {:?}", param);
    }
    //options for explorer and comparer
    let opt = param.to_options();
    //list of sources and destinations
    let mut src = Vec::new();
    let mut dst = Vec::new();
    //MPSC chanels
    //read threads to join thread
    let (to_join, from_read) = channel();
    //join thread to comp plus thread
    let (to_comp_m, from_join_m) = channel();
    //join thread to comp minus thread
    let (to_comp_p, from_join_p) = channel();
    //comp threads to write output
    let (to_script, from_comp) = channel();
    //channel to the logger
    let (to_logger, from_all) = channel();
    //channel to progression
    let (to_progress, from_all_prog) = channel();

    //start the logger
    let logfile = Path::new(&param.fic_out).with_extension("log"); //log is same a config except the extension
    let hlogger = start_thread_logger(from_all, logfile);

    let hprogress = start_thread_progress(from_all_prog, param.source.len());

    //start writer thread
    let hscriptgen = start_thread_scriptgen(
        from_comp,
        Path::new(&param.fic_out).to_path_buf(),
        to_logger.clone(),
        opt.verbose,
    );
    //get data for readers
    for s in param.source {
        src.push(Path::new(&s).to_path_buf());
    }
    for d in param.destination {
        dst.push(Path::new(&d).to_path_buf());
    }
    // start compare threads
    let hcompp = start_thread_comp_p(
        from_join_p,
        to_script.clone(),
        &opt,
        to_logger.clone(),
        to_progress.clone(),
    );
    let hcompm = start_thread_comp_m(
        from_join_m,
        to_script,
        &opt,
        to_logger.clone(),
        to_progress.clone(),
    );
    //start join thread
    let hjoin = start_thread_joiner(
        from_read,
        to_comp_m,
        to_comp_p,
        to_logger.clone(),
        opt.verbose,
        to_progress.clone(),
    );
    //start read threads
    let hreadsrc = start_thread_read_src(
        to_join.clone(),
        src,
        &opt,
        to_logger.clone(),
        to_progress.clone(),
    );
    let hreaddst = start_thread_read_dst(to_join, dst, &opt, to_logger, to_progress);

    //wait for threads to stop
    if hreadsrc.join().is_err() {
        println!("Thread {} finished with error", SRCREADER);
    }
    if hreaddst.join().is_err() {
        println!("Thread {} finished with error", DSTREADER);
    }
    if hjoin.join().is_err() {
        println!("Thread {} finished with error", JOINER);
    }
    if hcompp.join().is_err() {
        println!("Thread {} finished with error", COMPP);
    }
    if hcompm.join().is_err() {
        println!("Thread {} finished with error", COMPM);
    }
    if hscriptgen.join().is_err() {
        println!("Thread {} finished with error", SCRIPTGEN);
    }
    if hlogger.join().is_err() {
        println!("Thread {} finished with error", LOGGER);
    }
    if hprogress.join().is_err() {
        println!("Thread {} finished with error", PROGRESS);
    }
}
