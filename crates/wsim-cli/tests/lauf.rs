//! Headless runs with the real data, through the command line.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn wsim(args: &[&str]) -> Output {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Command::new(env!("CARGO_BIN_EXE_wsim"))
        .current_dir(root)
        .args(args)
        .output()
        .expect("wsim starts")
}

fn stdout(output: &Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn hash(output: &Output) -> String {
    let text = stdout(output);
    let line = text
        .lines()
        .find(|l| l.contains("Zustands-Hash"))
        .expect("hash line");
    line.rsplit(' ').next().unwrap().to_owned()
}

fn temp_file(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wsim-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

#[test]
fn shipped_data_validates() {
    let text = stdout(&wsim(&["validate", "data"]));
    assert!(text.ends_with("0 Fehler, 0 Warnungen.\n"), "{text}");
}

#[test]
fn same_seed_same_result_and_save_load_continue() {
    let common = ["run", "--seed", "17", "--runde", "monat", "--leise"];
    let straight = wsim(&[&common[..], &["--runden", "24"]].concat());

    let save = temp_file("halb.wsim");
    let save = save.to_str().unwrap();
    stdout(&wsim(
        &[&common[..], &["--runden", "12", "--speichern", save]].concat(),
    ));
    let continued = wsim(&[
        "run", "--laden", save, "--runde", "monat", "--runden", "12", "--leise",
    ]);

    assert_eq!(hash(&straight), hash(&continued));
    assert!(stdout(&continued).contains("Stand: 01.01.1902"));

    let other_seed = wsim(&[
        "run", "--seed", "18", "--runde", "monat", "--runden", "24", "--leise",
    ]);
    assert_ne!(hash(&straight), hash(&other_seed));
}

#[test]
fn reports_errors_in_german() {
    let output = wsim(&["run", "--startjahr", "1850"]);
    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr).trim(),
        "Das Startjahr 1850 ist nicht möglich; erlaubt sind 1900 bis 2026."
    );
}

#[test]
fn runs_until_a_date() {
    let text = stdout(&wsim(&["run", "--runde", "quartal", "--bis", "1901-01-01"]));
    assert_eq!(text.lines().filter(|l| l.ends_with("Tage)")).count(), 4);
    assert!(text.contains("Das Jahr 1901 beginnt."));
}

#[test]
fn world_runs_without_a_player_company() {
    // P0: world runs for balancing do not depend on a player who may go bankrupt.
    let args = [
        "run",
        "--nur-welt",
        "--ki",
        "3",
        "--bis",
        "1900-03-01",
        "--leise",
    ];
    let first = wsim(&args);
    assert_eq!(hash(&first), hash(&wsim(&args)));
    assert!(stdout(&first).contains("01.03.1900"));
}
