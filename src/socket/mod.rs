mod server;

use {
    crate::*,
    anyhow::{
        Context as _,
        Result,
    },
    std::{
        io::Write,
        os::unix::net::UnixStream,
    },
};

pub use server::Server;

pub fn send_action(
    context: &Context,
    action: &str,
) -> Result<()> {
    let path = context.unix_socket_path();
    let mut stream = UnixStream::connect(&path)
        .with_context(|| format!("Failed to connect to socket: {}", path.display()))?;
    stream.write_all(action.as_bytes())?;
    stream.flush()?;
    Ok(())
}
