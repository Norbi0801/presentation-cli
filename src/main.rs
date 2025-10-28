use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;

const FRAME_WIDTH: usize = 200;
const COLOR_ACCENT: &str = "\x1b[38;5;208m";
const COLOR_DIM: &str = "\x1b[38;5;94m";
const COLOR_GLOW: &str = "\x1b[38;5;159m";
const RESET: &str = "\x1b[0m";

fn main() {
    if let Err(error) = run() {
        eprintln!("\x1b[31mBłąd:\x1b[0m {}", error);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);

    let Some(path_arg) = args.next() else {
        print_usage();
        return Ok(());
    };

    let path = Path::new(&path_arg);
    let banner_arg = args
        .next()
        .unwrap_or_else(|| "presentations/banner.txt".to_string());
    let banner_path = Path::new(&banner_arg);

    display_banner(banner_path)?;
    println!();
    retro_separator("Rust Lab Terminal");
    println!(
        "{}SOURCE :: {}{}{}",
        COLOR_DIM,
        COLOR_ACCENT,
        path.display(),
        RESET
    );
    println!();

    let file = File::open(path)
        .map_err(|error| io::Error::new(error.kind(), format!("{}: {}", path.display(), error)))?;
    let reader = BufReader::new(file);

    print_frame_top();
    let mut rendered_anything = false;
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        rendered_anything = true;

        if index == 0 {
            wait_for_enter("Naciśnij Enter, aby rozpocząć prezentację...")?;
        } else {
            wait_for_enter("Enter -> kolejna linia")?;
        }

        transition_animation()?;
        println!();

        animate_line(index, line.trim_end_matches('\r'))?;
    }

    if !rendered_anything {
        print_empty_frame_message()?;
    }
    print_frame_bottom();

    if !rendered_anything {
        println!(
            "{}(Plik nie zawiera żadnych linii do zaprezentowania){}",
            COLOR_DIM, RESET
        );
    }

    println!();

    Ok(())
}

fn display_banner(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let banner = std::fs::read_to_string(path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("Baner ({}) nie został wczytany: {}", path.display(), error),
        )
    })?;

    crt_warmup()?;
    let mut stdout = io::stdout();

    for line in banner.lines() {
        println!("{}{}{}", COLOR_DIM, line, RESET);
        stdout.flush()?;
        thread::sleep(Duration::from_millis(70));
        print!("\x1b[1A\r{}{}{}\x1b[0K", COLOR_ACCENT, line, RESET);
        stdout.flush()?;
        println!();
        thread::sleep(Duration::from_millis(120));
    }

    thread::sleep(Duration::from_millis(320));
    Ok(())
}

fn wait_for_enter(message: &str) -> io::Result<()> {
    let mut stdout = io::stdout();
    print!("{}{}{} ", COLOR_DIM, message, RESET);
    stdout.flush()?;

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(())
}

fn transition_animation() -> io::Result<()> {
    let frames = [
        "[==>] forging slide",
        "[=>>] forging slide",
        "[>>>] forging slide",
        "[>>=] forging slide",
    ];
    let mut stdout = io::stdout();
    for frame in frames.iter().cycle().take(8) {
        print!("\r{}{}{}  ", COLOR_DIM, frame, RESET);
        stdout.flush()?;
        thread::sleep(Duration::from_millis(90));
    }

    print!("\r{}[OK] forge complete{}  ", COLOR_ACCENT, RESET);
    stdout.flush()?;
    thread::sleep(Duration::from_millis(260));
    print!("\r\x1b[0K");
    stdout.flush()?;
    Ok(())
}

fn animate_line(index: usize, text: &str) -> io::Result<()> {
    let mut stdout = io::stdout();
    let index_label = format!("{:03}", index + 1);
    let prefix = format!("| {}:: ", index_label);
    let available = FRAME_WIDTH.saturating_sub(prefix.len() + 1);

    print!("{}{}{}", COLOR_DIM, prefix, RESET);
    stdout.flush()?;

    let glyphs: Vec<char> = text.chars().collect();
    let mut printed = 0;
    for (i, ch) in glyphs.iter().enumerate() {
        if printed >= available {
            break;
        }

        if printed == available.saturating_sub(1) && i < glyphs.len() - 1 {
            print!("{}>{}", COLOR_ACCENT, RESET);
            stdout.flush()?;
            printed += 1;
            break;
        }

        print!("{}{}{}", COLOR_ACCENT, ch, RESET);
        stdout.flush()?;
        thread::sleep(Duration::from_millis(55));
        printed += 1;
    }

    let padding = available.saturating_sub(printed);
    if padding > 0 {
        print!("{}{}{}", COLOR_DIM, " ".repeat(padding), RESET);
    }
    print!("{}|{}", COLOR_DIM, RESET);
    println!();
    Ok(())
}

fn retro_separator(label: &str) {
    let label = format!(":: {} ::", label.to_uppercase());
    let fill = FRAME_WIDTH.saturating_sub(label.len());
    let left = fill / 2;
    let right = fill - left;

    println!(
        "{}{}{}{}{}{}{}",
        COLOR_DIM,
        "=".repeat(left),
        COLOR_GLOW,
        label,
        COLOR_DIM,
        "=".repeat(right),
        RESET
    );
}

fn print_frame_top() {
    println!("{}+{}+{}", COLOR_DIM, "-".repeat(FRAME_WIDTH - 2), RESET);
}

fn print_frame_bottom() {
    println!("{}+{}+{}", COLOR_DIM, "-".repeat(FRAME_WIDTH - 2), RESET);
}

fn print_empty_frame_message() -> io::Result<()> {
    let mut stdout = io::stdout();
    let prefix = "| --- :: ";
    let available = FRAME_WIDTH.saturating_sub(prefix.len() + 1);
    let message = "(brak treści w pliku)";
    let glyphs: Vec<char> = message.chars().collect();

    print!("{}{}{}", COLOR_DIM, prefix, RESET);
    stdout.flush()?;

    let mut printed = 0;
    for ch in glyphs.iter().take(available) {
        print!("{}{}{}", COLOR_DIM, ch, RESET);
        stdout.flush()?;
        printed += 1;
    }

    let padding = available.saturating_sub(printed);
    if padding > 0 {
        print!("{}{}{}", COLOR_DIM, " ".repeat(padding), RESET);
    }
    print!("{}|{}", COLOR_DIM, RESET);
    println!();
    Ok(())
}

fn crt_warmup() -> io::Result<()> {
    let mut stdout = io::stdout();
    let phases = [
        "[.. ] spinning up retro tube",
        "[<. ] calibrating scanline",
        "[<<.] loading rust pigment",
        "[<<<] ready to beam",
    ];

    for phase in &phases {
        print!("\r{}{}{}", COLOR_DIM, phase, RESET);
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
        Domyślny baner: presentations/banner.txt\n\
        Przykład: cargo run -- presentations/example.txt"
    );
}
