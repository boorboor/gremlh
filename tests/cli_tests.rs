use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

// =============================================================================
// Helpers
// =============================================================================

fn setup_env() -> TempDir {
    TempDir::new().expect("Failed to create temp dir")
}

fn create_file(dir: &TempDir, filename: &str, content: &[u8]) {
    let path = dir.path().join(filename);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut file = File::create(&path).unwrap();
    file.write_all(content).unwrap();
}

fn get_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_gremlh"))
}

// =============================================================================
// Core Functionality Tests
// =============================================================================

#[test]
fn test_file_dirty_scan_and_bidi_detection() -> Result<(), Box<dyn std::error::Error>> {
    let env = setup_env();
    create_file(&env, "dirty.txt", "H\u{200B}ello\u{202E}World!".as_bytes());

    let mut cmd = get_cmd();
    cmd.arg(env.path());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("dirty.txt:1:2: found"))
        .stderr(predicate::str::contains("Zero Width Space"))
        .stderr(predicate::str::contains("dirty.txt:1:7: found"))
        .stderr(predicate::str::contains("Security Risk!"))
        .stderr(predicate::str::contains("2 gremlins found"));

    Ok(())
}

#[test]
fn test_clean_run_is_silent_success() -> Result<(), Box<dyn std::error::Error>> {
    let env = setup_env();
    create_file(&env, "clean.txt", b"Hello World!");
    create_file(&env, "src/main.rs", b"fn main() {}");

    let mut cmd = get_cmd();
    cmd.arg(env.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    Ok(())
}

// =============================================================================
// Write/Fix Tests (--write)
// =============================================================================

#[test]
fn test_file_dirty_write_atomic_replacement() -> Result<(), Box<dyn std::error::Error>> {
    let env = setup_env();
    let file_path = env.path().join("fixme.txt");

    create_file(&env, "fixme.txt", "\u{201C}Hello\u{201D}".as_bytes());

    let mut cmd = get_cmd();
    cmd.arg(&file_path).arg("--write");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("fixme.txt:1:1: found"));

    let content = std::fs::read_to_string(&file_path)?;
    assert_eq!(content, "\"Hello\"\n");

    Ok(())
}

#[test]
fn test_character_removal_vs_replacement_write() -> Result<(), Box<dyn std::error::Error>> {
    let env = setup_env();
    let file_path = env.path().join("mixed.txt");

    create_file(
        &env,
        "mixed.txt",
        "\u{FEFF}A\u{200B}B\u{00A0}C\u{037E}".as_bytes(),
    );

    let mut cmd = get_cmd();
    cmd.arg(&file_path).arg("--write");

    cmd.assert()
        .failure()
        .stderr(
            predicate::str::contains("mixed.txt:1:1: found").and(predicate::str::contains("BOM")),
        )
        .stderr(
            predicate::str::contains("mixed.txt:1:3: found")
                .and(predicate::str::contains("Zero Width Space")),
        )
        .stderr(
            predicate::str::contains("mixed.txt:1:5: found").and(predicate::str::contains("NBSP")),
        )
        .stderr(
            predicate::str::contains("mixed.txt:1:7: found")
                .and(predicate::str::contains("Greek Question Mark")),
        );

    let content = std::fs::read_to_string(&file_path)?;
    assert_eq!(content, "AB C;\n");

    Ok(())
}

#[cfg(unix)]
#[test]
fn test_permissions_preserved() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;

    let env = setup_env();
    let file_path = env.path().join("executable_script.sh");

    create_file(
        &env,
        "executable_script.sh",
        "#!/bin/bash\necho \"\u{201C}Hello\u{201D}\"".as_bytes(),
    );

    let perms = fs::metadata(&file_path)?.permissions();
    let new_mode = (perms.mode() & !0o777) | 0o755;
    fs::set_permissions(&file_path, Permissions::from_mode(new_mode))?;

    let mode_before = fs::metadata(&file_path)?.permissions().mode();
    assert_eq!(mode_before & 0o777, 0o755);

    let mut cmd = get_cmd();
    cmd.arg(&file_path).arg("--write").arg("--verbose");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("warning:").not());

    let content = std::fs::read_to_string(&file_path)?;
    assert!(content.contains("echo \"\"Hello\"\""));

    let mode_after = fs::metadata(&file_path)?.permissions().mode();
    assert_eq!(mode_after & 0o777, 0o755);

    Ok(())
}

// =============================================================================
// STDIN Tests
// =============================================================================

#[test]
fn test_stdin_detection_dirty() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = get_cmd();

    cmd.write_stdin("H\u{00A0}ello");

    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("H ello"))
        .stderr(predicate::str::contains("<stdin>:1:2: found"))
        .stderr(predicate::str::contains("gremlins found").not());

    Ok(())
}

#[test]
fn test_stdin_clean() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = get_cmd();
    let clean_input = "Hello World\nThis is clean.";
    cmd.write_stdin(clean_input);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(clean_input))
        .stderr(predicate::str::is_empty());

    Ok(())
}

// =============================================================================
// Traversal and Ignore Logic Tests
// =============================================================================

#[test]
fn test_gitignore_is_respected() -> Result<(), Box<dyn std::error::Error>> {
    let env = setup_env();

    fs::create_dir(env.path().join(".git")).unwrap();

    create_file(
        &env,
        "node_modules/bad.js",
        "var x = '\u{200B}';".as_bytes(),
    );
    create_file(&env, ".gitignore", b"node_modules/");

    let mut cmd = get_cmd();
    cmd.arg(env.path());

    cmd.assert().success().stderr(predicate::str::is_empty());

    let mut cmd_override = get_cmd();
    cmd_override.arg(env.path()).arg("--no-ignore");
    cmd_override
        .assert()
        .failure()
        .stderr(predicate::str::contains("bad.js:1:10: found"));

    Ok(())
}

#[test]
fn test_hidden_files() -> Result<(), Box<dyn std::error::Error>> {
    let env = setup_env();

    create_file(&env, ".env", "SECRET=\u{200B}VALUE".as_bytes());
    create_file(
        &env,
        ".config/settings.toml",
        "key = \"\u{201C}value\u{201D}\"".as_bytes(),
    );

    let mut cmd = get_cmd();
    cmd.arg(env.path());
    cmd.assert().success().stderr(predicate::str::is_empty());

    let mut cmd_hidden = get_cmd();
    cmd_hidden.arg(env.path()).arg("--hidden");
    cmd_hidden
        .assert()
        .failure()
        .stderr(predicate::str::contains(".env:1:8:"))
        .stderr(predicate::str::contains("settings.toml:1:8:"));

    Ok(())
}

#[test]
fn test_binary_files_skipped() -> Result<(), Box<dyn std::error::Error>> {
    let env = setup_env();

    let mut content = Vec::new();
    content.extend_from_slice(b"DATA");
    content.push(0x00);
    content.extend_from_slice("BINARY\u{200B}".as_bytes());

    create_file(&env, "data.bin", &content);
    create_file(&env, "clean.txt", b"Hello");

    let mut cmd = get_cmd();
    cmd.arg(env.path());
    cmd.assert().success().stderr(predicate::str::is_empty());

    let mut cmd_verbose = get_cmd();
    cmd_verbose.arg(env.path()).arg("--verbose");

    cmd_verbose
        .assert()
        .success()
        .stderr(predicate::str::contains("binary:").and(predicate::str::contains("data.bin")))
        .stderr(predicate::str::contains("clean:").and(predicate::str::contains("clean.txt")))
        .stderr(predicate::str::contains("No gremlins found."));

    Ok(())
}

// =============================================================================
// Error Handling and Verbosity Tests
// =============================================================================

#[test]
fn test_non_existent_path_error() -> Result<(), Box<dyn std::error::Error>> {
    let bad_path = "i_do_not_exist_12345";

    let mut cmd = get_cmd();
    cmd.arg(bad_path);

    cmd.assert().failure().stderr(
        predicate::str::contains("error:").and(predicate::str::contains("Path does not exist")),
    );

    Ok(())
}

#[test]
fn test_verbose_mode_output_files() -> Result<(), Box<dyn std::error::Error>> {
    let env = setup_env();
    create_file(&env, "clean.txt", b"Hello");
    create_file(&env, "dirty.txt", "H\u{200B}i".as_bytes());

    let mut cmd = get_cmd();
    cmd.arg(env.path()).arg("--verbose");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Scanning path:"))
        .stderr(predicate::str::contains("clean:").and(predicate::str::contains("clean.txt")))
        .stderr(predicate::str::contains("dirty.txt:1:2: found"))
        .stderr(predicate::str::contains("1 gremlins found"));

    Ok(())
}

#[test]
fn test_verbose_mode_output_stdin() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = get_cmd();
    cmd.arg("--verbose");
    cmd.write_stdin("H\u{00A0}ello");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Reading from STDIN..."))
        .stderr(predicate::str::contains("<stdin>:1:2: found"));

    Ok(())
}

// =============================================================================
// Concurrency Tests
// =============================================================================

#[test]
fn test_multiple_files_output_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let env = setup_env();

    create_file(
        &env,
        "file1.txt",
        "File1\u{200B}Line1\nFile1\u{200B}Line2".as_bytes(),
    );
    create_file(
        &env,
        "file2.txt",
        "File2\u{200B}Line1\nFile2\u{200B}Line2".as_bytes(),
    );
    create_file(&env, "file3.txt", "File3\u{200B}Line1".as_bytes());

    let mut cmd = get_cmd();
    cmd.arg(env.path());
    cmd.arg("-j").arg("4");

    let output = cmd.assert().failure().get_output().clone();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stderr.contains("file1.txt:1:6:"));
    assert!(stderr.contains("file1.txt:2:6:"));
    assert!(stderr.contains("file2.txt:1:6:"));
    assert!(stderr.contains("file2.txt:2:6:"));
    assert!(stderr.contains("file3.txt:1:6:"));
    assert!(stderr.contains("5 gremlins found"));

    Ok(())
}
