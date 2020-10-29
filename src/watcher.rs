use {
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


pub struct Watcher {
    pub receiver: Receiver<()>,
}

impl Watcher {
    pub fn new() -> Result<Self> {
        let src_dir = env::current_dir()?.join("src");
        if !src_dir.exists() {
            return Err(anyhow!("src directory not found"));
        }
        let mut inotify = Inotify::init()?;
        inotify.add_watch(
            src_dir,
            WatchMask::MODIFY | WatchMask::CREATE | WatchMask::DELETE,
        )?;
        // TODO add Cargo.toml
        let (sender, receiver) = bounded(0);
        thread::spawn(move || {
            let mut buffer = [0; 1024];
            loop {
                match inotify.read_events_blocking(&mut buffer) {
                    Ok(_) => {
                        debug!("inotify event received");
                        if let Err(e) = sender.send(()) {
                            debug!("error when notifying on inotify event: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Error in inotify read_events_blocking: {}", e);
                        return;
                    }
                };
            }
            debug!("closing watcher");
        });
        Ok(Self {
            receiver,
        })
    }
}

