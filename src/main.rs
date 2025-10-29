use std::env;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use clap::{Parser, ValueEnum};
use dotenvy::dotenv;

mod interaction;
mod theme;
mod watch;

use crate::interaction::run_presentation;
use crate::theme::{self, ThemePalette};
use crate::watch::watch_file;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const ITALIC: &str = "\x1b[3m";
const UNDERLINE: &str = "\x1b[4m";

#[derive(Parser, Debug)]
#[command(
    author = "RustLab",
    version,
    about = "Retro-futurystyczny silnik prezentacyjny dla terminala",
    disable_help_subcommand = true
)]
struct Cli {
    /// Plik z treścią prezentacji
    script: PathBuf,
    /// Ścieżka do pliku baneru ASCII
    #[arg(short, long)]
    banner: Option<PathBuf>,
    /// Nadpisanie tytułu prezentacji
    #[arg(short, long)]
    title: Option<String>,
    /// Nadpisanie szerokości ramki
    #[arg(long)]
    frame_width: Option<usize>,
    /// Wybór motywu kolorystycznego
    #[arg(long, value_enum)]
    theme: Option<ThemeName>,
    /// Ścieżka do pliku motywu w formacie TOML
    #[arg(long)]
    theme_path: Option<PathBuf>,
    /// Natychmiastowe renderowanie (bez animacji)
    #[arg(long)]
    instant: bool,
    /// Pominięcie baneru startowego
    #[arg(long)]
    skip_banner: bool,
    /// Obserwowanie pliku i automatyczne odświeżanie prezentacji
    #[arg(long)]
    watch: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
#[clap(rename_all = "kebab_case")]
enum ThemeName {
    Neon,
    Amber,
    Arctic,
}

impl ThemeName {
    fn defaults(self) -> ThemePalette {
        match self {
            ThemeName::Neon => {
                ThemePalette::new("\x1b[38;5;214m", "\x1b[38;5;238m", "\x1b[38;5;51m")
            }
            ThemeName::Amber => {
                ThemePalette::new("\x1b[38;5;178m", "\x1b[38;5;94m", "\x1b[38;5;221m")
            }
            ThemeName::Arctic => {
                ThemePalette::new("\x1b[38;5;195m", "\x1b[38;5;250m", "\x1b[38;5;117m")
            }
        }
    }
}

impl fmt::Display for ThemeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ThemeName::Neon => "neon",
            ThemeName::Amber => "amber",
            ThemeName::Arctic => "arctic",
        };
        write!(f, "{}", name.to_uppercase())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Config {
    frame_width: usize,
    palette: ThemePalette,
    banner_path: Option<PathBuf>,
    presentation_title: String,
    theme_label: String,
    animations_enabled: bool,
}

impl Config {
    fn from_sources(cli: &Cli) -> Result<Self, Box<dyn std::error::Error>> {
        let (theme_label, defaults) = if let Some(path) = cli.theme_path.as_deref() {
            let spec = theme::load_from_path(path)?;
            (spec.label().to_string(), spec.palette().clone())
        } else {
            let theme = cli
                .theme
                .or_else(|| {
                    env::var("PRESENTATION_THEME")
                        .ok()
                        .and_then(|value| ThemeName::from_str(&value, true).ok())
                })
                .unwrap_or(ThemeName::Neon);

            (theme.to_string(), theme.defaults())
        };

        let palette = ThemePalette::new(
            env::var("COLOR_ACCENT").unwrap_or_else(|_| defaults.accent().to_string()),
            env::var("COLOR_DIM").unwrap_or_else(|_| defaults.dim().to_string()),
            env::var("COLOR_GLOW").unwrap_or_else(|_| defaults.glow().to_string()),
        );

        let frame_width = cli
            .frame_width
            .or_else(|| {
                env::var("FRAME_WIDTH")
                    .ok()
                    .and_then(|value| value.parse().ok())
            })
            .unwrap_or(120);

        let presentation_title = cli
            .title
            .clone()
            .or_else(|| env::var("PRESENTATION_TITLE").ok())
            .unwrap_or_else(|| "Rust Lab Terminal".to_string());

        let default_banner = env::var("DEFAULT_BANNER_PATH")
            .unwrap_or_else(|_| "presentations/banner.txt".to_string());
        let banner_path = if cli.skip_banner {
            None
        } else {
            Some(
                cli.banner
                    .clone()
                    .unwrap_or_else(|| PathBuf::from(default_banner)),
            )
        };

        Ok(Self {
            frame_width,
            palette,
            banner_path,
            presentation_title,
            theme_label,
            animations_enabled: !cli.instant,
        })
    }

    pub(crate) fn frame_width(&self) -> usize {
        self.frame_width
    }

    pub(crate) fn color_accent(&self) -> &str {
        self.palette.accent()
    }

    pub(crate) fn color_dim(&self) -> &str {
        self.palette.dim()
    }

    pub(crate) fn color_glow(&self) -> &str {
        self.palette.glow()
    }

    fn banner_path(&self) -> Option<&Path> {
        self.banner_path.as_deref()
    }

    fn presentation_title(&self) -> &str {
        &self.presentation_title
    }

    fn theme_label(&self) -> &str {
        &self.theme_label
    }

    pub(crate) fn animations_enabled(&self) -> bool {
        self.animations_enabled
    }

    pub(crate) fn pause(&self, duration: Duration) {
        if self.animations_enabled {
            thread::sleep(duration);
        }
    }

    pub(crate) fn adjust_frame_width(&mut self, delta: isize) -> bool {
        let current = self.frame_width as isize;
        let updated = (current + delta).max(40) as usize;
        if updated != self.frame_width {
            self.frame_width = updated;
            return true;
        }
        false
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Segment {
    kind: SegmentKind,
}

#[derive(Debug, Clone)]
pub(crate) enum SegmentKind {
    Heading(String),
    Bullet(String),
    Callout(String),
    Plain(String),
    Separator,
}

impl Segment {
    fn new(kind: SegmentKind) -> Self {
        Self { kind }
    }

    pub(crate) fn kind(&self) -> &SegmentKind {
        &self.kind
    }
}

fn parse_segments<R: BufRead>(reader: R) -> io::Result<Vec<Segment>> {
    let mut segments = Vec::new();
    for line in reader.lines() {
        let line = line?;
        segments.push(classify_segment(&line));
    }
    Ok(segments)
}

fn classify_segment(line: &str) -> Segment {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Segment::new(SegmentKind::Plain(String::new()));
    }

    if trimmed.len() >= 3 && trimmed.chars().all(|ch| matches!(ch, '-' | '–' | '=')) {
        return Segment::new(SegmentKind::Separator);
    }

    if trimmed.starts_with('#') {
        let content = trimmed.trim_start_matches('#').trim();
        if !content.is_empty() {
            return Segment::new(SegmentKind::Heading(content.to_string()));
        }
    }

    if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
        let content = trimmed[2..].trim_start();
        return Segment::new(SegmentKind::Bullet(content.to_string()));
    }

    if trimmed.starts_with('>') {
        let content = trimmed.trim_start_matches('>').trim_start();
        return Segment::new(SegmentKind::Callout(content.to_string()));
    }

    Segment::new(SegmentKind::Plain(trimmed.to_string()))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("\x1b[31mBłąd:\x1b[0m {}", error);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let cli = Cli::parse();
    let script_path = cli.script.clone();
    let mut config = Config::from_sources(&cli)?;

    if let Some(banner_path) = config.banner_path() {
        display_banner(&config, banner_path)?;
        println!();
    }

    present_script(&mut config, &script_path)?;

    if cli.watch {
        println!(
            "{}WATCH :: {}{}aktywny tryb nasłuchiwania (Ctrl+C aby zakończyć){}",
            config.color_dim(),
            BOLD,
            config.color_accent(),
            RESET
        );

        let debounce = Duration::from_millis(250);
        let watch_path = script_path.clone();

        watch_file(&watch_path, debounce, || {
            println!();
            println!(
                "{}SYNC  :: {}{}wykryto zmianę, odświeżanie prezentacji{}",
                config.color_dim(),
                BOLD,
                config.color_glow(),
                RESET
            );

            if let Err(error) = present_script(&mut config, &watch_path) {
                eprintln!("\x1b[31mBłąd:\x1b[0m {error}");
            }

            true
        })
        .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;
    }

    Ok(())
}

fn present_script(
    config: &mut Config,
    script_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    retro_separator(config, config.presentation_title());
    print_session_meta(config, script_path);

    let file = File::open(script_path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("{}: {}", script_path.display(), error),
        )
    })?;
    let reader = BufReader::new(file);
    let segments = parse_segments(reader)?;

    if segments.is_empty() {
        print_frame_top(config);
        print_empty_frame_message(config)?;
        print_frame_bottom(config);
        println!(
            "{}⚠ {}{}Brak treści do wyświetlenia{}",
            config.color_dim(),
            config.color_accent(),
            ITALIC,
            RESET
        );
        println!();
        return Ok(());
    }

    run_presentation(config, &segments)?;

    println!();

    Ok(())
}

fn display_banner(config: &Config, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let banner = std::fs::read_to_string(path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("Baner ({}) nie został wczytany: {}", path.display(), error),
        )
    })?;

    crt_warmup(config)?;
    let mut stdout = io::stdout();

    for line in banner.lines() {
        if config.animations_enabled() {
            println!("{}{}{}", config.color_dim(), line, RESET);
            stdout.flush()?;
            config.pause(Duration::from_millis(60));
            print!(
                "\x1b[1A\r{}{}{}{}\x1b[0K",
                config.color_glow(),
                BOLD,
                line,
                RESET
            );
            stdout.flush()?;
            println!();
            config.pause(Duration::from_millis(110));
        } else {
            println!("{}{}{}{}", config.color_glow(), BOLD, line, RESET);
        }
    }

    config.pause(Duration::from_millis(240));
    Ok(())
}

pub(crate) fn transition_animation(config: &Config) -> io::Result<()> {
    if !config.animations_enabled() {
        return Ok(());
    }

    let frames = [
        "[⠁] synchronizacja torów",
        "[⠃] kalibracja światła",
        "[⠇] ładowanie wektorów",
        "[⠇] montaż kadrów",
        "[⠧] strojenie luminancji",
        "[⠷] finalizacja",
    ];
    let mut stdout = io::stdout();
    for frame in frames.iter().cycle().take(10) {
        print!("\r{}{}{}  ", config.color_dim(), frame, RESET);
        stdout.flush()?;
        config.pause(Duration::from_millis(70));
    }

    print!("\r{}{}[GOTOWE]{}");
    stdout.flush()?;
    config.pause(Duration::from_millis(210));
    print!("\r\x1b[0K");
    stdout.flush()?;
    Ok(())
}

pub(crate) fn animate_line(
    config: &Config,
    index: usize,
    segment: &Segment,
    animate: bool,
) -> io::Result<()> {
    let mut stdout = io::stdout();
    let index_label = format!("{:03}", index + 1);
    let prefix = format!("│ {} :: ", index_label);
    let available = config.frame_width().saturating_sub(prefix.len() + 1);

    print!("{}{}{}", config.color_dim(), prefix, RESET);
    stdout.flush()?;

    if let SegmentKind::Separator = segment.kind() {
        let fill = "─".repeat(available);
        print!("{}{}{}", config.color_dim(), fill, RESET);
        print!("{}│{}", config.color_dim(), RESET);
        println!();
    } else {
        let (display_text, color, style_prefix, delay) = match segment.kind() {
            SegmentKind::Heading(text) => (
                text.to_uppercase(),
                config.color_glow(),
                Some(format!("{}{}", BOLD, UNDERLINE)),
                Duration::from_millis(35),
            ),
            SegmentKind::Bullet(text) => (
                format!("• {}", text),
                config.color_accent(),
                None,
                Duration::from_millis(45),
            ),
            SegmentKind::Callout(text) => (
                format!("❝ {} ❞", text),
                config.color_glow(),
                Some(ITALIC.to_string()),
                Duration::from_millis(38),
            ),
            SegmentKind::Plain(text) => (
                text.to_string(),
                if text.is_empty() {
                    config.color_dim()
                } else {
                    config.color_accent()
                },
                None,
                Duration::from_millis(55),
            ),
            SegmentKind::Separator => unreachable!(),
        };

        let style_prefix_ref = style_prefix.as_deref().unwrap_or("");
        let glyphs: Vec<char> = display_text.chars().collect();
        let mut printed = 0;

        if available > 0 && (!glyphs.is_empty() || !style_prefix_ref.is_empty()) {
            if !style_prefix_ref.is_empty() {
                print!("{}", style_prefix_ref);
            }
            print!("{}", color);
            stdout.flush()?;

            if animate && config.animations_enabled() {
                for (i, ch) in glyphs.iter().enumerate() {
                    if printed >= available {
                        break;
                    }

                    if printed == available.saturating_sub(1) && i < glyphs.len() - 1 {
                        print!("›");
                        stdout.flush()?;
                        printed += 1;
                        break;
                    }

                    print!("{}", ch);
                    stdout.flush()?;
                    config.pause(delay);
                    printed += 1;
                }
            } else {
                let mut buffer = String::new();
                for (i, ch) in glyphs.iter().enumerate() {
                    if printed >= available {
                        break;
                    }

                    if printed == available.saturating_sub(1) && i < glyphs.len() - 1 {
                        buffer.push('›');
                        printed += 1;
                        break;
                    }

                    buffer.push(*ch);
                    printed += 1;
                }
                print!("{}", buffer);
            }

            print!("{}", RESET);
        }

        let padding = available.saturating_sub(printed);
        if padding > 0 {
            print!("{}{}{}", config.color_dim(), " ".repeat(padding), RESET);
        }
        print!("{}│{}", config.color_dim(), RESET);
        println!();
    }

    Ok(())
}

fn print_session_meta(config: &Config, script_path: &Path) {
    println!(
        "{}SOURCE :: {}{}{}{}",
        config.color_dim(),
        BOLD,
        config.color_accent(),
        script_path.display(),
        RESET
    );
    println!(
        "{}THEME  :: {}{}{}{}  {}FRAME :: {}{}{}{}  {}MODE :: {}{}{}{}",
        config.color_dim(),
        BOLD,
        config.color_glow(),
        config.theme_label().to_uppercase(),
        RESET,
        config.color_dim(),
        BOLD,
        config.color_accent(),
        config.frame_width(),
        RESET,
        config.color_dim(),
        BOLD,
        config.color_accent(),
        if config.animations_enabled() {
            "CINEMATIC"
        } else {
            "INSTANT"
        },
        RESET
    );
    println!();
}

fn retro_separator(config: &Config, label: &str) {
    let label = format!("╢ {} ╟", label.to_uppercase());
    let fill = config.frame_width().saturating_sub(label.len());
    let left = fill / 2;
    let right = fill - left;

    println!(
        "{}{}{}{}{}{}{}",
        config.color_dim(),
        "═".repeat(left),
        config.color_glow(),
        label,
        config.color_dim(),
        "═".repeat(right),
        RESET
    );
}

pub(crate) fn print_frame_top(config: &Config) {
    println!(
        "{}╭{}╮{}",
        config.color_dim(),
        "─".repeat(config.frame_width().saturating_sub(2)),
        RESET
    );
}

pub(crate) fn print_frame_bottom(config: &Config) {
    println!(
        "{}╰{}╯{}",
        config.color_dim(),
        "─".repeat(config.frame_width().saturating_sub(2)),
        RESET
    );
}

fn print_empty_frame_message(config: &Config) -> io::Result<()> {
    let mut stdout = io::stdout();
    let prefix = "│ SYS :: ";
    let available = config.frame_width().saturating_sub(prefix.len() + 1);
    let message = "(brak treści w pliku)";
    let glyphs: Vec<char> = message.chars().collect();

    print!("{}{}{}", config.color_dim(), prefix, RESET);
    stdout.flush()?;

    let mut printed = 0;
    for ch in glyphs.iter().take(available) {
        print!("{}{}{}", ITALIC, config.color_dim(), ch);
        stdout.flush()?;
        printed += 1;
    }
    print!("{}", RESET);

    let padding = available.saturating_sub(printed);
    if padding > 0 {
        print!("{}{}{}", config.color_dim(), " ".repeat(padding), RESET);
    }
    print!("{}│{}", config.color_dim(), RESET);
    println!();
    Ok(())
}

fn crt_warmup(config: &Config) -> io::Result<()> {
    if !config.animations_enabled() {
        return Ok(());
    }

    let mut stdout = io::stdout();
    let phases = [
        "[.. ] spinning up retro tube",
        "[<. ] calibrating scanline",
        "[<<.] loading rust pigment",
        "[<<<] ready to beam",
    ];

    for phase in &phases {
        print!("\r{}{}{}", config.color_dim(), phase, RESET);
        stdout.flush()?;
        config.pause(Duration::from_millis(220));
    }

    print!("\r\x1b[0K");
    stdout.flush()?;
    Ok(())
}
