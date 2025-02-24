use {
    crate::*,
    anyhow::Result,
    termimad::crossterm::{
        cursor,
        execute,
        terminal,
    },
};

/// Move the curstor to the x, y position
pub fn goto(
    w: &mut W,
    x: u16,
    y: u16,
) -> Result<()> {
    execute!(w, cursor::MoveTo(x, y))?;
    Ok(())
}

/// Move the curstor to the start of the provided line
pub fn goto_line(
    w: &mut W,
    y: u16,
) -> Result<()> {
    execute!(w, cursor::MoveTo(0, y))?;
    Ok(())
}

/// Clear from the current position to the end of the line
pub fn clear_line(w: &mut W) -> Result<()> {
    execute!(w, terminal::Clear(terminal::ClearType::UntilNewLine))?;
    Ok(())
}
