use {
    crate::*,
    anyhow::{
        Context as _,
        Result,
    },
    std::{
        fs,
        io::{
            BufRead,
            BufReader,
        },
        os::unix::net::UnixListener,
        path::PathBuf,
        thread,
    },
    termimad::crossbeam::channel::Sender,
};

pub struct Server {
    path: PathBuf,
}

impl Server {
    pub fn new(
        context: &Context,
        tx: Sender<Action>,
    ) -> Result<Self> {
        let path = context.unix_socket_path();
        if fs::metadata(&path).is_ok() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to remove socket file {}", &path.display()))?;
        }
        let listener = UnixListener::bind(&path)?;
        info!("listening on {}", path.display());
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else {
                    warn!("error while accepting connection");
                    continue;
                };
                let tx = tx.clone();
                thread::spawn(move || {
                    debug!("new connection");
                    let mut br = BufReader::new(&stream);
                    let mut line = String::new();
                    while br.read_line(&mut line).is_ok() {
                        while line.ends_with('\n') || line.ends_with('\r') {
                            line.pop();
                        }
                        debug!("line => {:?}", &line);
                        if line.is_empty() {
                            debug!("empty line, closing connection");
                            break;
                        }
                        match line.parse() {
                            Ok(action) => {
                                if tx.send(action).is_err() {
                                    error!("failed to send action");
                                }
                            }
                            Err(e) => {
                                warn!("failed to parse action: {e}");
                            }
                        }
                        line.clear();
                    }
                    debug!("closed connection");
                });
            }
        });
        Ok(Self { path })
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        debug!("removing socket file");
        let _ = fs::remove_file(&self.path);
    }
}
