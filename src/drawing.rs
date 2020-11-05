use {
    crate::*,
    anyhow::*,
    crossterm::{cursor, execute, terminal},
    std::io::Write,
};

pub fn goto(w: &mut W, y: u16) -> Result<()> {
    execute!(
        w,
        cursor::MoveTo(0, y),
        terminal::Clear(terminal::ClearType::CurrentLine)
    )?;
    Ok(())
}
