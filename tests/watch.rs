use std::fs;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use RustLabPresentations::watch::watch_file;

#[test]
fn triggers_callback_after_file_change() {
    let temp_dir = tempfile::tempdir().expect("temporary directory");
    let script_path = temp_dir.path().join("presentation.txt");

    fs::write(&script_path, "Pierwsza wersja").expect("initial content");

    let (tx, rx) = mpsc::channel();
    let watch_path = script_path.clone();

    let handle = thread::spawn(move || {
        watch_file(&watch_path, Duration::from_millis(100), move || {
            tx.send(()).expect("send notification");
            false
        })
        .expect("watch to complete");
    });

    // Give the watcher a moment to initialise before mutating the file.
    thread::sleep(Duration::from_millis(200));

    fs::write(&script_path, "Zmieniona treść").expect("updated content");

    rx.recv_timeout(Duration::from_secs(2))
        .expect("watcher callback to be triggered");

    handle.join().expect("watch thread to finish");
}
