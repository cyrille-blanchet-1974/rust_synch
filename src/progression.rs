use std::sync::mpsc::Receiver;
use std::thread::{spawn, JoinHandle};

pub struct Progression {
    total: usize,
    actual: usize,
}

//reads worth 10
//joins worth 1
//compares worth 2

pub enum Action {
    Read,
    Join,
    Compare,
}

fn value(a: Action) -> usize {
    match a {
        Action::Read => 5,
        Action::Join => 1,
        Action::Compare => 2,
    }
}

impl Progression {
    pub fn new(nb_src: usize) -> Progression {
        Progression {
            total: nb_src
                * (2 * (value(Action::Read) + value(Action::Join) + value(Action::Compare))),
            actual: 0,
        }
    }

    pub fn run(&mut self, a: Action) {
        self.actual += value(a);
        self.show();
    }

    pub fn show(&self) {
        let percent = (self.actual * 100) / self.total;
        println!("{}%", percent);
    }
}

/**
* start progression thread
*
*/
pub fn start_thread_progress(from_all: Receiver<Action>, nb_src: usize) -> JoinHandle<()> {
    spawn(move || {
        let mut p = Progression::new(nb_src);
        for data in from_all {
            p.run(data);
        }
    })
}
