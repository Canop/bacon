use {
    crate::*,
    anyhow::*,
    crossterm::{cursor, execute, terminal},
    std::io::Write,
};

pub fn goto(w: &mut W, y: u16) -> Result<()> {
    execute!(w, cursor::MoveTo(0, y))?;
    Ok(())
}
pub fn clear_line(w: &mut W) -> Result<()> {
    execute!(w, terminal::Clear(terminal::ClearType::UntilNewLine))?;
    Ok(())
}
