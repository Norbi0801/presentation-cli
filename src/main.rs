use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;

use dotenvy::dotenv;

const RESET: &str = "\x1b[0m";

#[derive(Debug, Clone)]
struct Config {
    frame_width: usize,
    color_accent: String,
    color_dim: String,
    color_glow: String,
    default_banner_path: String,
    presentation_title: String,
}

impl Config {
    fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let frame_width = env::var("FRAME_WIDTH").unwrap_or_else(|_| "200".to_string());
        let color_accent =
            env::var("COLOR_ACCENT").unwrap_or_else(|_| "\x1b[38;5;208m".to_string());
        let color_dim = env::var("COLOR_DIM").unwrap_or_else(|_| "\x1b[38;5;94m".to_string());
        let color_glow = env::var("COLOR_GLOW").unwrap_or_else(|_| "\x1b[38;5;159m".to_string());
        let default_banner_path = env::var("DEFAULT_BANNER_PATH")
            .unwrap_or_else(|_| "presentations/banner.txt".to_string());
        let presentation_title =
            env::var("PRESENTATION_TITLE").unwrap_or_else(|_| "Rust Lab Terminal".to_string());

        Ok(Self {
            frame_width: frame_width.parse::<usize>()?,
            color_accent,
            color_dim,
            color_glow,
            default_banner_path,
            presentation_title,
        })
    }

    fn frame_width(&self) -> usize {
        self.frame_width
    }

    fn color_accent(&self) -> &str {
        &self.color_accent
    }

    fn color_dim(&self) -> &str {
        &self.color_dim
    }

    fn color_glow(&self) -> &str {
        &self.color_glow
    }

    fn default_banner_path(&self) -> &str {
        &self.default_banner_path
    }

    fn presentation_title(&self) -> &str {
        &self.presentation_title
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("\x1b[31mBłąd:\x1b[0m {}", error);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let config = Config::from_env()?;

    let mut args = env::args().skip(1);

    let Some(path_arg) = args.next() else {
        print_usage();
        return Ok(());
    };

    let path = Path::new(&path_arg);
    let banner_arg = args
        .next()
        .unwrap_or_else(|| config.default_banner_path().to_string());
    let banner_path = Path::new(&banner_arg);

    display_banner(&config, banner_path)?;
    println!();
    retro_separator(&config, config.presentation_title());
    println!(
        "{}SOURCE :: {}{}{}",
        config.color_dim(),
        config.color_accent(),
        path.display(),
        RESET
    );
    println!();

    let file = File::open(path)
        .map_err(|error| io::Error::new(error.kind(), format!("{}: {}", path.display(), error)))?;
    let reader = BufReader::new(file);

    print_frame_top(&config);
    let mut rendered_anything = false;
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        rendered_anything = true;

        if index == 0 {
            wait_for_enter(&config, "Naciśnij Enter, aby rozpocząć prezentację...")?;
        } else {
            wait_for_enter(&config, "Enter -> kolejna linia")?;
        }

        transition_animation(&config)?;
        println!();

        animate_line(&config, index, line.trim_end_matches('\r'))?;
    }

    if !rendered_anything {
        print_empty_frame_message(&config)?;
    }
    print_frame_bottom(&config);

    if !rendered_anything {
        println!(
            "{}(Plik nie zawiera żadnych linii do zaprezentowania){}",
            config.color_dim(),
            RESET
        );
    }

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
        println!("{}{}{}", config.color_dim(), line, RESET);
        stdout.flush()?;
        thread::sleep(Duration::from_millis(70));
        print!("\x1b[1A\r{}{}{}\x1b[0K", config.color_accent(), line, RESET);
        stdout.flush()?;
        println!();
        thread::sleep(Duration::from_millis(120));
    }

    thread::sleep(Duration::from_millis(320));
    Ok(())
}

fn wait_for_enter(config: &Config, message: &str) -> io::Result<()> {
    let mut stdout = io::stdout();
    print!("{}{}{} ", config.color_dim(), message, RESET);
    stdout.flush()?;

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(())
}

fn transition_animation(config: &Config) -> io::Result<()> {
    let frames = [
        "[==>] forging slide",
        "[=>>] forging slide",
        "[>>>] forging slide",
        "[>>=] forging slide",
    ];
    let mut stdout = io::stdout();
    for frame in frames.iter().cycle().take(8) {
        print!("\r{}{}{}  ", config.color_dim(), frame, RESET);
        stdout.flush()?;
        thread::sleep(Duration::from_millis(90));
    }

    print!("\r{}[OK] forge complete{}  ", config.color_accent(), RESET);
    stdout.flush()?;
    thread::sleep(Duration::from_millis(260));
    print!("\r\x1b[0K");
    stdout.flush()?;
    Ok(())
}

fn animate_line(config: &Config, index: usize, text: &str) -> io::Result<()> {
    let mut stdout = io::stdout();
    let index_label = format!("{:03}", index + 1);
    let prefix = format!("| {}:: ", index_label);
    let available = config.frame_width().saturating_sub(prefix.len() + 1);

    print!("{}{}{}", config.color_dim(), prefix, RESET);
    stdout.flush()?;

    let glyphs: Vec<char> = text.chars().collect();
    let mut printed = 0;
    for (i, ch) in glyphs.iter().enumerate() {
        if printed >= available {
            break;
        }

        if printed == available.saturating_sub(1) && i < glyphs.len() - 1 {
            print!("{}>{}", config.color_accent(), RESET);
            stdout.flush()?;
            printed += 1;
            break;
        }

        print!("{}{}{}", config.color_accent(), ch, RESET);
        stdout.flush()?;
        thread::sleep(Duration::from_millis(55));
        printed += 1;
    }

    let padding = available.saturating_sub(printed);
    if padding > 0 {
        print!("{}{}{}", config.color_dim(), " ".repeat(padding), RESET);
    }
    print!("{}|{}", config.color_dim(), RESET);
    println!();
    Ok(())
}

fn retro_separator(config: &Config, label: &str) {
    let label = format!(":: {} ::", label.to_uppercase());
    let fill = config.frame_width().saturating_sub(label.len());
    let left = fill / 2;
    let right = fill - left;

    println!(
        "{}{}{}{}{}{}{}",
        config.color_dim(),
        "=".repeat(left),
        config.color_glow(),
        label,
        config.color_dim(),
        "=".repeat(right),
        RESET
    );
}

fn print_frame_top(config: &Config) {
    println!(
        "{}+{}+{}",
        config.color_dim(),
        "-".repeat(config.frame_width().saturating_sub(2)),
        RESET
    );
}

fn print_frame_bottom(config: &Config) {
    println!(
        "{}+{}+{}",
        config.color_dim(),
        "-".repeat(config.frame_width().saturating_sub(2)),
        RESET
    );
}

fn print_empty_frame_message(config: &Config) -> io::Result<()> {
    let mut stdout = io::stdout();
    let prefix = "| --- :: ";
    let available = config.frame_width().saturating_sub(prefix.len() + 1);
    let message = "(brak treści w pliku)";
    let glyphs: Vec<char> = message.chars().collect();

    print!("{}{}{}", config.color_dim(), prefix, RESET);
    stdout.flush()?;

    let mut printed = 0;
    for ch in glyphs.iter().take(available) {
        print!("{}{}{}", config.color_dim(), ch, RESET);
        stdout.flush()?;
        printed += 1;
    }

    let padding = available.saturating_sub(printed);
    if padding > 0 {
        print!("{}{}{}", config.color_dim(), " ".repeat(padding), RESET);
    }
    print!("{}|{}", config.color_dim(), RESET);
    println!();
    Ok(())
}

fn crt_warmup(config: &Config) -> io::Result<()> {
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
        thread::sleep(Duration::from_millis(240));
    }

    print!("\r\x1b[0K");
    stdout.flush()?;
    Ok(())
}

fn print_usage() {
    println!(
        "Użycie: cargo run -- <ścieżka_do_pliku> [ścieżka_banera]\n\
        Domyślny baner: wartość DEFAULT_BANNER_PATH z pliku .env\n\
        Przykład: cargo run -- presentations/example.txt"
    );
}
