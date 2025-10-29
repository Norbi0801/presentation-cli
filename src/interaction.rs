use std::io::{self, Stdout, Write};
use std::time::Instant;

use crossterm::ExecutableCommand;
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{self, Clear, ClearType};

use crate::{
    BOLD, Config, ITALIC, RESET, Segment, SegmentKind, Slide, print_frame_bottom, print_frame_top,
    render_segment, transition_animation,
};

const FRAME_WIDTH_STEP: isize = 2;

pub(crate) fn run_presentation(config: &mut Config, slides: &[Slide]) -> io::Result<()> {
    if slides.is_empty() {
        return Ok(());
    }

    let mut stdout = io::stdout();
    stdout.flush()?;
    let start_row = cursor::position().map(|(_, row)| row).unwrap_or(0);
    let origin = (0, start_row);

    let _raw_mode = RawModeGuard::new()?;

    let start_time = Instant::now();
    render(&mut stdout, origin, config, slides, 0, true, start_time)?;
    let mut current_index = 0usize;

    loop {
        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Left => {
                    if current_index > 0 {
                        current_index -= 1;
                        render(
                            &mut stdout,
                            origin,
                            config,
                            slides,
                            current_index,
                            true,
                            start_time,
                        )?;
                    }
                }
                KeyCode::Right | KeyCode::Enter => {
                    if current_index + 1 < slides.len() {
                        current_index += 1;
                        render(
                            &mut stdout,
                            origin,
                            config,
                            slides,
                            current_index,
                            true,
                            start_time,
                        )?;
                    } else {
                        break;
                    }
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => break,
                KeyCode::Char('+') | KeyCode::Char('=') => {
                    if config.adjust_frame_width(FRAME_WIDTH_STEP) {
                        render(
                            &mut stdout,
                            origin,
                            config,
                            slides,
                            current_index,
                            false,
                            start_time,
                        )?;
                    }
                }
                KeyCode::Char('-') | KeyCode::Char('_') => {
                    if config.adjust_frame_width(-FRAME_WIDTH_STEP) {
                        render(
                            &mut stdout,
                            origin,
                            config,
                            slides,
                            current_index,
                            false,
                            start_time,
                        )?;
                    }
                }
                KeyCode::Esc => break,
                _ => {}
            },
            Event::Resize(_, _) => {
                render(
                    &mut stdout,
                    origin,
                    config,
                    slides,
                    current_index,
                    false,
                    start_time,
                )?;
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
    slides: &[Slide],
    index: usize,
    animate: bool,
    start_time: Instant,
) -> io::Result<()> {
    stdout.execute(cursor::MoveTo(origin.0, origin.1))?;
    stdout.execute(Clear(ClearType::FromCursorDown))?;

    if animate && config.animations_enabled() {
        transition_animation(config)?;
        println!();
    }

    print_frame_top(config);
    render_slide(config, index, &slides[index], animate)?;
    print_frame_bottom(config);
    println!();
    print_instructions(config, index, slides.len(), &slides[index]);
    if config.presenter_mode() {
        print_presenter_panel(config, &slides[index], index, slides.len(), start_time);
    }
    stdout.flush()?;

    Ok(())
}

fn render_slide(
    config: &Config,
    slide_index: usize,
    slide: &Slide,
    animate: bool,
) -> io::Result<()> {
    if slide.segments().is_empty() {
        let placeholder =
            Segment::new(SegmentKind::Plain("(tylko notatki prelegenta)".to_string()));
        render_segment(config, slide_index, 0, &placeholder, animate)?;
        return Ok(());
    }

    for (line_index, segment) in slide.segments().iter().enumerate() {
        render_segment(config, slide_index, line_index, segment, animate)?;
    }

    Ok(())
}

fn print_instructions(config: &Config, index: usize, total: usize, slide: &Slide) {
    println!(
        "{}CTRL ::{} {}←/→{} lub Enter sekwencje  {}+/-{} szerokość  {}Q/Esc{} wyjście  {}SEQ ::{} {}{:03}/{:03}{}  {}DECK ::{} {}{:02}{}  {}LOCAL ::{} {}{:02}{}  {}FRAME ::{} {}{}{}",
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
        slide.deck_index() + 1,
        RESET,
        config.color_dim(),
        RESET,
        config.color_accent(),
        slide.index_in_source(),
        RESET,
        config.color_dim(),
        RESET,
        config.color_accent(),
        config.frame_width(),
        RESET
    );
}

fn print_presenter_panel(
    config: &Config,
    slide: &Slide,
    index: usize,
    total: usize,
    start_time: Instant,
) {
    let elapsed = start_time.elapsed();
    let minutes = elapsed.as_secs() / 60;
    let seconds = elapsed.as_secs() % 60;
    let source_label = slide
        .source()
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_string())
        .unwrap_or_else(|| slide.source().display().to_string());

    println!(
        "{}PANEL ::{} {}{:02}:{:02}{} elapsed  {}SEQ ::{} {}{:03}/{:03}{}  {}DECK ::{} {}{:02}{}  {}LOCAL ::{} {}{:02}{}  {}SOURCE ::{} {}{}{}",
        config.color_dim(),
        RESET,
        config.color_glow(),
        minutes,
        seconds,
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
        slide.deck_index() + 1,
        RESET,
        config.color_dim(),
        RESET,
        config.color_accent(),
        slide.index_in_source(),
        RESET,
        config.color_dim(),
        RESET,
        config.color_accent(),
        source_label,
        RESET
    );

    if slide.notes().is_empty() {
        println!(
            "{}NOTES ::{} {}{}(brak notatek){}",
            config.color_dim(),
            RESET,
            config.color_dim(),
            ITALIC,
            RESET
        );
    } else {
        println!("{}NOTES ::{}", config.color_dim(), RESET);
        for (note_index, note) in slide.notes().iter().enumerate() {
            println!(
                "  {}{}{:02}{} {}{}{}",
                config.color_dim(),
                BOLD,
                note_index + 1,
                RESET,
                config.color_accent(),
                note,
                RESET
            );
        }
    }
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
