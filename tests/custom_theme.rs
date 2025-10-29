use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn runs_with_custom_theme_path() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("--instant")
        .arg("--skip-banner")
        .arg("--theme-path")
        .arg("themes/nebula.toml")
        .arg("tests/fixtures/empty.txt");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("NEBULA"));

    Ok(())
}
