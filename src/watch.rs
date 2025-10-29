use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use notify::event::EventKind;
use notify::{RecursiveMode, Watcher, recommended_watcher};

/// Watches the provided file path for changes and triggers the callback after
/// a debounce period. The callback should return `true` to keep watching or
/// `false` to stop.
pub fn watch_file<F>(path: &Path, debounce: Duration, mut on_change: F) -> notify::Result<()>
where
    F: FnMut() -> bool,
{
    let target = path.to_path_buf();
    let canonical_target = canonicalize_path(&target);
    let (tx, rx) = mpsc::channel();

    let mut watcher = recommended_watcher(move |event| {
        let _ = tx.send(event);
    })?;

    watcher.watch(&target, RecursiveMode::NonRecursive)?;

    let mut last_trigger = Instant::now();
    let mut first_event = true;

    for received in rx {
        let event = match received {
            Ok(event) => event,
            Err(error) => {
                eprintln!("Błąd obserwatora: {error}");
                continue;
            }
        };

        if !is_relevant_event(&event.paths, &target, &canonical_target) {
            continue;
        }

        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {}
            EventKind::Remove(_) => {}
            _ => continue,
        }

        let now = Instant::now();
        if !first_event && now.duration_since(last_trigger) < debounce {
            continue;
        }

        first_event = false;
        last_trigger = now;

        if !on_change() {
            break;
        }
    }

    Ok(())
}

fn canonicalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn is_relevant_event(paths: &[PathBuf], original: &Path, canonical: &Path) -> bool {
    paths
        .iter()
        .any(|candidate| match candidate.canonicalize() {
            Ok(candidate_canonical) => candidate_canonical == canonical,
            Err(_) => candidate == canonical || candidate == original,
        })
}
