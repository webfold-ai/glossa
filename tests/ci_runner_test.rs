//! CI Runner Simulation & Workflow Invariant Test Suite.
//!
//! Verifies that ptyktos-lingo satisfies all CI/CD runner requirements:
//! 1. Exits with non-zero code (1) when translations are missing or corrupted.
//! 2. Emits GitHub Actions workflow command annotations (`::error title=...::`).
//! 3. Generates markdown table step summaries in `$GITHUB_STEP_SUMMARY`.
//! 4. Catches Apeiron Null-Gate violations (e.g. dropped `{slot}` or tag corruptions) before deployment.
//! 5. Successfully passes with exit code 0 when coverage is 100%.

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
    assert!(
        !check_output.status.success(),
        "CI check must fail with non-zero exit code when translations are missing"
    );
    assert_eq!(
        check_output.status.code(),
        Some(1),
        "CI check must return exit code 1"
    );

    // Verify GitHub Workflow Annotation output
    let stdout = String::from_utf8_lossy(&check_output.stdout);
    assert!(
        stdout.contains("::error title=Missing Translation"),
        "Must emit GitHub Actions error annotation command"
    );

    // Verify GITHUB_STEP_SUMMARY markdown was written
    assert!(
        summary_file.exists(),
        "GITHUB_STEP_SUMMARY file must be created"
    );
    let summary_md = fs::read_to_string(&summary_file).unwrap();
    assert!(summary_md.contains("ptyktos-lingo CI Check: FAILED"));
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

    assert!(
        check_output.status.success(),
        "CI check must pass with exit code 0 when coverage is complete"
    );

    let summary_md = fs::read_to_string(&summary_file).unwrap();
    assert!(summary_md.contains("ptyktos-lingo CI Check: PASSED"));
}

/// MUTATION CONTROL for the Integrity Gate wiring.
///
/// Before `check` invoked the gate, a lockfile with COMPLETE coverage but a
/// CORRUPT translation passed CI with exit code 0 -- while printing
/// "Validated by Zero-Fault Integrity Gate". This test is the regression that
/// must turn red if the gate call in `Commands::Check` is ever removed again.
///
/// It writes a lockfile by hand (rather than translating) so the corruption is
/// guaranteed: coverage is 100%, and the Spanish translation is missing the
/// `{user}` slot and carries an unbalanced tag.
///
/// Real-world class (lingo.dev #863, lingo.dev #2159, inlang #3801): a
/// translation that exists, is not missing, and is nonetheless broken.
#[test]
fn test_ci_runner_failure_on_corrupt_existing_translation() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        src_dir.join("Greeting.tsx"),
        "export default function Greeting({ user }) { return <h1>Welcome, {user.name}!</h1>; }",
    )
    .unwrap();

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

    let extract = Command::new(bin_path)
        .arg("--dir")
        .arg(root)
        .arg("extract")
        .output()
        .unwrap();
    assert!(extract.status.success());

    // Hand-corrupt the lockfile: full coverage, broken translation.
    let lock_path = root.join("lingo.lock");
    let lock_raw = fs::read_to_string(&lock_path).unwrap();
    let lock_val: serde_json::Value =
        serde_json::from_str(&lock_raw).expect("lockfile must be valid JSON");
    let entries = lock_val["entries"].as_object_mut().expect("entries map");

    let mut corrupted_any = false;
    for entry in entries.values_mut() {
        let src = entry["canonical_text"].as_str().unwrap_or_default().to_string();
        if !src.contains("{user}") {
            continue;
        }
        // Drop the {user} slot and break the markup balance.
        entry["locales"]["es"]["translated_text"] =
            serde_json::Value::String("<0>Bienvenido.</0".to_string());
        corrupted_any = true;
    }
    assert!(corrupted_any, "fixture must contain a {{user}} term to corrupt");

    fs::write(&lock_path, serde_json::to_string_pretty(&lock_val).unwrap()).unwrap();

    // Coverage is complete -- so only the gate can fail this.
    let summary_file = root.join("step_summary.md");
    let check_output = Command::new(bin_path)
        .arg("--dir")
        .arg(root)
        .arg("check")
        .arg("--ci")
        .arg("--format")
        .arg("github")
        .env("GITHUB_ACTIONS", "true")
        .env("GITHUB_STEP_SUMMARY", &summary_file)
        .output()
        .unwrap();

    assert!(
        !check_output.status.success(),
        "CI check MUST fail when an existing translation drops a placeholder, \
         even though coverage is 100%. This is the mutation control for the \
         Integrity Gate wiring in Commands::Check."
    );

    let stdout = String::from_utf8_lossy(&check_output.stdout).to_string();
    assert!(
        stdout.contains("Translation Integrity"),
        "expected a GitHub Actions annotation titled 'Translation Integrity', got: {stdout}"
    );
    assert!(
        stdout.contains("::error"),
        "expected a ::error workflow annotation, got: {stdout}"
    );

    let summary_md = fs::read_to_string(&summary_file).unwrap();
    assert!(
        summary_md.contains("Integrity Gate"),
        "step summary must report the gate result, got: {summary_md}"
    );
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
