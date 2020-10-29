use {
    crate::*,
    anyhow::*,
    crossbeam::channel::{
        Receiver,
        Sender,
        select,
        unbounded,
    },
    crossterm::{
        cursor,
        execute,
        ExecutableCommand,
        event::{
            KeyModifiers,
            KeyEvent,
            KeyCode::*,
        },
        QueueableCommand,
        style::{Colorize, Styler},
        terminal,
    },
    std::{
        env,
        io::Write,
        path::PathBuf,
    },
    termimad::{Event, EventSource},
};

pub fn run(
    w: &mut W,
) -> Result<()> {
    let mut state = AppState::new()?;
    let event_source = EventSource::new()?;
    let user_events = event_source.receiver();
    debug!("launched app");
    state.draw(w)?;
    state.report = Some(Report::compute()?);
    state.computing = false;
    state.draw(w)?;
    let watcher = Watcher::new()?;
    let computer = Computer::new()?;
    loop {
        select! {
            recv(user_events) -> user_event => {
                match user_event? {
                    Event::Resize(width, height) => {
                        state.screen = (width, height);
                        state.draw(w)?;
                    }
                    Event::Key(KeyEvent{ code, modifiers }) => {
                        match (code, modifiers) {
                            (Char('q'), KeyModifiers::NONE) | (Char('c'), KeyModifiers::CONTROL) => {
                                debug!("user requests quit");
                                break;
                            }
                            _ => {
                                debug!("ignored key event: {:?}", user_event);
                            }
                        }
                    }
                    _ => {
                    }
                }
                event_source.unblock(false);
            }
            recv(watcher.receiver) -> _ => {
                debug!("got a watcher event");
                if let Err(e) = computer.task_sender.try_send(()) {
                    debug!("error sending task: {}", e);
                } else {
                    state.computing = true;
                    state.draw(w)?;
                }
            }
            recv(computer.report_receiver) -> report => {
                state.report = Some(report?);
                state.computing = false;
                state.draw(w)?;
            }
        }
    }
    Ok(())
}
