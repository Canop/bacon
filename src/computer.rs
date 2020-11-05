use {
    crate::*,
    anyhow::*,
    crossbeam::channel::{bounded, Receiver, Sender},
    std::{path::PathBuf, thread},
};

/// a computer calling `cargo watch` in a separate
/// thread when asked to and sending the results
/// in a channel.
///
/// Channel sizes are designed to avoid useless
/// computations.
pub struct Computer {
    pub task_sender: Sender<()>,
    pub report_receiver: Receiver<Report>,
}

impl Computer {
    pub fn new(root_dir: PathBuf, use_clippy: bool) -> Result<Self> {
        let (task_sender, task_receiver) = bounded(0);
        let (report_sender, report_receiver) = bounded(1);
        thread::spawn(move || {
            for _ in task_receiver {
                match Report::compute(&root_dir, use_clippy) {
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
