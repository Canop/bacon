use {
    crate::*,
    anyhow::*,
    crossbeam::channel::{
        bounded,
        Receiver,
        Sender,
        select,
        unbounded,
    },
    inotify::{
        Inotify,
        WatchMask,
    },
    std::{
        env,
        thread,
    },
};


pub struct Computer {
    pub task_sender: Sender<()>,
    pub report_receiver: Receiver<Report>,
}

impl Computer {
    pub fn new() -> Result<Self> {
        let (task_sender, task_receiver) = bounded(0);
        let (report_sender, report_receiver) = bounded(1);
        thread::spawn(move || {
            for _ in task_receiver {
                debug!("COMPILER got task");
                match Report::compute() {
                    Ok(report) => {
                        if let Err(e) = report_sender.send(report) {
                            debug!("error when sending report: {}", e);
                            break;
                        }
                    }
                    Err(err) => {
                        warn!("error in computing report: {}", err);
                    }
                }
            }
            debug!("closing computer");
        });
        Ok(Self {
            task_sender,
            report_receiver,
        })
    }
}


