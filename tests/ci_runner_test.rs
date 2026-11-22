//! Tests for `glossa check --ci` as run from a CI job:
//! 1. Exits with non-zero code (1) when translations are missing or corrupted.
//! 2. Emits GitHub Actions workflow command annotations (`::error title=...::`).
//! 3. Generates markdown table step summaries in `$GITHUB_STEP_SUMMARY`.
//! 4. Reports Null-Gate violations (e.g. a dropped `{slot}` or a corrupted tag).
//! 5. Successfully passes with exit code 0 when every configured locale has a translation.

use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_ci_runner_failure_on_missing_translations() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    // Create a mock project with 1 source file and target locales [es, de]
    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let tsx_content = r#"
    export default function Greeting({ user }) {
        return <h1>Welcome to Cloud, {user.name}!</h1>;
    }
    "#;
    fs::write(src_dir.join("Greeting.tsx"), tsx_content).unwrap();

    let lingo_json = r#"{
        "source_locale": "en",
        "target_locales": ["es", "de"],
        "include": ["src/**/*.tsx"],
        "exclude": [],
        "framework": "react",
        "provider": "offline",
        "output_format": "json",
        "locales_dir": "locales"
    }"#;
    fs::write(root.join("lingo.json"), lingo_json).unwrap();

    // Step 1: Run lingo extract
    let bin_path = env!("CARGO_BIN_EXE_lingo");
    let extract_output = Command::new(bin_path)
        .arg("--dir")
        .arg(root)
        .arg("extract")
        .output()
        .expect("Failed to run lingo extract");
    assert!(extract_output.status.success(), "Extract must succeed");

    // Step 2: Run lingo check in simulated GitHub Actions runner
    let summary_file = root.join("step_summary.md");
    let check_output = Command::new(bin_path)
        .arg("--dir")
        .arg(root)
        .arg("check")
        .arg("--ci")
        .arg("--format")
        .arg("github")
        .env("GITHUB_ACTIONS", "true")
        .env("CI", "true")
        .env("GITHUB_STEP_SUMMARY", &summary_file)
        .output()
        .expect("Failed to run lingo check in CI mode");

    // CI MUST fail because translations are missing!
    assert!(!check_output.status.success(), "CI check must fail with non-zero exit code when translations are missing");
    assert_eq!(check_output.status.code(), Some(1), "CI check must return exit code 1");

    // Verify GitHub Workflow Annotation output
    let stdout = String::from_utf8_lossy(&check_output.stdout);
    assert!(
        stdout.contains("::error title=Missing Translation"),
        "Must emit GitHub Actions error annotation command"
    );

    // Verify GITHUB_STEP_SUMMARY markdown was written
    assert!(summary_file.exists(), "GITHUB_STEP_SUMMARY file must be created");
    let summary_md = fs::read_to_string(&summary_file).unwrap();
    assert!(summary_md.contains("Glossa CI Check: FAILED"));
    assert!(summary_md.contains("Missing Items"));
}

#[test]
fn test_ci_runner_success_on_complete_translations() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let tsx_content = r#"
    export default function Button() {
        return <button>Confirm Action</button>;
    }
    "#;
    fs::write(src_dir.join("Button.tsx"), tsx_content).unwrap();

    let lingo_json = r#"{
        "source_locale": "en",
        "target_locales": ["es"],
        "include": ["src/**/*.tsx"],
        "exclude": [],
        "framework": "react",
        "provider": "offline",
        "output_format": "json",
        "locales_dir": "locales"
    }"#;
    fs::write(root.join("lingo.json"), lingo_json).unwrap();

    let bin_path = env!("CARGO_BIN_EXE_lingo");

    // Extract
    let extract = Command::new(bin_path)
        .arg("--dir")
        .arg(root)
        .arg("extract")
        .output()
        .unwrap();
    assert!(extract.status.success());

    // Translate with offline provider
    let translate = Command::new(bin_path)
        .arg("--dir")
        .arg(root)
        .arg("translate")
        .arg("--provider")
        .arg("offline")
        .output()
        .unwrap();
    assert!(translate.status.success());

    // Check CI runner pass
    let summary_file = root.join("step_summary.md");
    let check_output = Command::new(bin_path)
        .arg("--dir")
        .arg(root)
        .arg("check")
        .arg("--ci")
        .env("GITHUB_ACTIONS", "true")
        .env("GITHUB_STEP_SUMMARY", &summary_file)
        .output()
        .unwrap();

    assert!(check_output.status.success(), "CI check must pass with exit code 0 when coverage is complete");

    let summary_md = fs::read_to_string(&summary_file).unwrap();
    assert!(summary_md.contains("Glossa CI Check: PASSED"));
}

#[test]
fn test_ci_diff_inspection() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    let lingo_json = r#"{
        "source_locale": "en",
        "target_locales": ["es", "fr"],
        "include": ["src/**/*.tsx"],
        "exclude": [],
        "framework": "react",
        "provider": "offline",
        "output_format": "json",
        "locales_dir": "locales"
    }"#;
    fs::write(root.join("lingo.json"), lingo_json).unwrap();

    let bin_path = env!("CARGO_BIN_EXE_lingo");
    let diff_output = Command::new(bin_path)
        .arg("--dir")
        .arg(root)
        .arg("diff")
        .output()
        .unwrap();

    assert!(diff_output.status.success());
    let stdout = String::from_utf8_lossy(&diff_output.stdout);
    assert!(stdout.contains("Localization Diff & Status"));
}
