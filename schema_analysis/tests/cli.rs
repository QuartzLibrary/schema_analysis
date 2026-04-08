#![cfg(feature = "cli")]

use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;

fn input(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/cli_fixtures/input")
        .join(name)
}

fn cmd() -> Command {
    Command::cargo_bin("schema_analysis").unwrap()
}

// ── Input formats ──────────────────────────────────────────────────

#[test]
fn json_file_schema() {
    cmd()
        .arg(input("sample.json"))
        .assert()
        .success()
        .stdout(include_str!("cli_fixtures/expected/json_schema.json"));
}

#[test]
fn yaml_file_schema() {
    cmd()
        .arg(input("sample.yaml"))
        .assert()
        .success()
        .stdout(include_str!("cli_fixtures/expected/yaml_schema.json"));
}

#[test]
fn xml_file_schema() {
    cmd()
        .arg(input("sample.xml"))
        .assert()
        .success()
        .stdout(include_str!("cli_fixtures/expected/xml_schema.json"));
}

#[test]
fn toml_file_schema() {
    cmd()
        .arg(input("sample.toml"))
        .assert()
        .success()
        .stdout(include_str!("cli_fixtures/expected/toml_schema.json"));
}

// ── Output modes ───────────────────────────────────────────────────

#[test]
fn json_output_rust() {
    cmd()
        .arg(input("sample.json"))
        .args(["--output", "rust"])
        .assert()
        .success()
        .stdout(include_str!("cli_fixtures/expected/json_rust.rs"));
}

#[test]
fn json_output_typescript() {
    cmd()
        .arg(input("sample.json"))
        .args(["--output", "typescript"])
        .assert()
        .success()
        .stdout(include_str!("cli_fixtures/expected/json_typescript.ts"));
}

#[test]
fn json_output_json_schema() {
    cmd()
        .arg(input("sample.json"))
        .args(["--output", "json-schema"])
        .assert()
        .success()
        .stdout(include_str!("cli_fixtures/expected/json_json_schema.json"));
}

// ── Flags ──────────────────────────────────────────────────────────

#[test]
fn compact_flag() {
    cmd()
        .arg(input("sample.json"))
        .arg("--compact")
        .assert()
        .success()
        .stdout(include_str!(
            "cli_fixtures/expected/json_schema_compact.json"
        ));
}

#[test]
fn minimal_flag() {
    cmd()
        .arg(input("sample.json"))
        .arg("--minimal")
        .assert()
        .success()
        .stdout(include_str!(
            "cli_fixtures/expected/json_schema_minimal.json"
        ));
}

// ── Stdin ──────────────────────────────────────────────────────────

#[test]
fn stdin_json() {
    cmd()
        .write_stdin(r#"{"name": "Alice", "age": 30, "active": true, "scores": [95, 87, 92]}"#)
        .assert()
        .success()
        .stdout(include_str!("cli_fixtures/expected/stdin_json_schema.json"));
}

// ── Multi-file merging ─────────────────────────────────────────────

#[test]
fn merge_multiple_json_files() {
    cmd()
        .arg(input("sample.json"))
        .arg(input("sample2.json"))
        .assert()
        .success()
        .stdout(include_str!(
            "cli_fixtures/expected/json_merged_schema.json"
        ));
}

// ── Error cases ────────────────────────────────────────────────────

#[test]
fn unknown_extension_fails() {
    cmd()
        .arg("nonexistent.xyz")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unrecognized file extension"));
}

#[test]
fn missing_file_fails() {
    cmd()
        .arg("nonexistent.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to open"));
}

#[test]
fn no_extension_fails() {
    cmd()
        .arg("noext")
        .assert()
        .failure()
        .stderr(predicate::str::contains("has no extension"));
}

// ── Format override ────────────────────────────────────────────────

#[test]
fn format_flag_overrides_extension() {
    // Parse a YAML file but force TOML format — should fail because YAML isn't valid TOML
    cmd()
        .arg(input("sample.yaml"))
        .args(["--format", "toml"])
        .assert()
        .failure();
}
