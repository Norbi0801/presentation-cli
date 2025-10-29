use std::io::{self, Stdout, Write};

use crossterm::ExecutableCommand;
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{self, Clear, ClearType};

use crate::{
    Config, RESET, Segment, animate_line, print_frame_bottom, print_frame_top, transition_animation,
};

const FRAME_WIDTH_STEP: isize = 2;

pub(crate) fn run_presentation(config: &mut Config, segments: &[Segment]) -> io::Result<()> {
    if segments.is_empty() {
        return Ok(());
    }

    let mut stdout = io::stdout();
    stdout.flush()?;
    let start_row = cursor::position().map(|(_, row)| row).unwrap_or(0);
    let origin = (0, start_row);

    let _raw_mode = RawModeGuard::new()?;

    render(&mut stdout, origin, config, segments, 0, true)?;
    let mut current_index = 0usize;

    loop {
        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Left => {
                    if current_index > 0 {
                        current_index -= 1;
                        render(&mut stdout, origin, config, segments, current_index, true)?;
                    }
                }
                KeyCode::Right | KeyCode::Enter => {
                    if current_index + 1 < segments.len() {
                        current_index += 1;
                        render(&mut stdout, origin, config, segments, current_index, true)?;
                    } else {
                        break;
                    }
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => break,
                KeyCode::Char('+') | KeyCode::Char('=') => {
                    if config.adjust_frame_width(FRAME_WIDTH_STEP) {
                        render(&mut stdout, origin, config, segments, current_index, false)?;
                    }
                }
                KeyCode::Char('-') | KeyCode::Char('_') => {
                    if config.adjust_frame_width(-FRAME_WIDTH_STEP) {
                        render(&mut stdout, origin, config, segments, current_index, false)?;
                    }
                }
                KeyCode::Esc => break,
                _ => {}
            },
            Event::Resize(_, _) => {
                render(&mut stdout, origin, config, segments, current_index, false)?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn render(
    stdout: &mut Stdout,
    origin: (u16, u16),
    config: &Config,
    segments: &[Segment],
    index: usize,
    animate: bool,
) -> io::Result<()> {
    stdout.execute(cursor::MoveTo(origin.0, origin.1))?;
    stdout.execute(Clear(ClearType::FromCursorDown))?;

    if animate && config.animations_enabled() {
        transition_animation(config)?;
        println!();
    }

    print_frame_top(config);
    animate_line(config, index, &segments[index], animate)?;
    print_frame_bottom(config);
    println!();
    print_instructions(config, index, segments.len());
    stdout.flush()?;

    Ok(())
}

fn print_instructions(config: &Config, index: usize, total: usize) {
    println!(
        "{}CTRL ::{} {}←/→{} lub Enter sekwencje  {}+/-{} szerokość  {}Q/Esc{} wyjście  {}SEQ ::{} {}{:03}/{:03}{}  {}FRAME ::{} {}{}{}",
        config.color_dim(),
        RESET,
        config.color_glow(),
        RESET,
        config.color_glow(),
        RESET,
        config.color_glow(),
        RESET,
        config.color_dim(),
        RESET,
        config.color_accent(),
        index + 1,
        total,
        RESET,
        config.color_dim(),
        RESET,
        config.color_accent(),
        config.frame_width(),
        RESET
    );
}

struct RawModeGuard;

impl RawModeGuard {
    fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}
