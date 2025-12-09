# Contributing to Gremlin Hunter (gremlh)

Thank you for your interest in contributing to `gremlh`!

We welcome contributions of all forms: bug reports, feature requests, documentation improvements, and code changes.

## 🛠️ Development Setup

`gremlh` is written in Rust. You will need a recent stable version of Rust installed.

1.  **Clone the repo**
    ```bash
    git clone [https://github.com/boorboor/gremlh.git](https://github.com/boorboor/gremlh.git)
    cd gremlh
    ```

2.  **Install development tools**
    We use [prek](https://github.com/j178/prek) to manage pre-commit hooks (formatting and linting).
    ```bash
    cargo install prek
    prek install
    ```

3.  **Build and Run**
    ```bash
    cargo build
    # Run against a specific file/dir
    cargo run -- ./src
    ```

## 🏗️ Architecture Overview

`gremlh` is designed for speed using a parallel **Producer-Consumer** architecture:

1.  **Scanner (Producer):** Uses the `ignore` crate (`WalkBuilder`) to traverse directories in parallel, respecting `.gitignore`.
2.  **Processor:** Each thread reads files, scans for gremlins, and (optionally) writes to a temporary file.
3.  **Reporter (Consumer):** Results are sent via an `mpsc` channel to a single printer thread to ensure `stderr` output remains ordered and doesn't interleave.
4.  **Stats:** Statistics are tracked using `AtomicUsize` shared across threads.

If you are working on the scanning logic, look in `src/scanner.rs`. If you are working on file IO or threading, look in `src/processor.rs`.

## 🧪 Testing

We have a robust integration test suite in `tests/cli_tests.rs` using `assert_cmd`.

* **Run all tests:**
    ```bash
    cargo test
    ```

* **Writing new tests:**
    If you add a new feature or flag, please add an integration test in `tests/cli_tests.rs`. Use the `setup_env()` helper to create a safe, isolated temporary directory for your test files.

    ```rust
    #[test]
    fn test_my_new_feature() -> Result<(), Box<dyn std::error::Error>> {
        let env = setup_env();
        create_file(&env, "test.txt", b"content");

        let mut cmd = get_cmd();
        cmd.arg(env.path()).arg("--my-new-flag");

        cmd.assert().success();
        Ok(())
    }
    ```

## 🧩 How to Add a New Gremlin

To add detection for a new invisible character or homoglyph:

1.  Open `src/definitions.rs`.
2.  Add a match arm to `identify_gremlin`.
3.  Define the `GremlinAction`:
    * **Description:** Short, human-readable name.
    * **Replacement:** `None` (delete) or `Some('c')` (replace).

**Example:**
```rust
'\u{1234}' => Some(GremlinAction {
    description: "New Invisible Separator",
    replacement: Some(' '),
}),
```

## 🚀 Release Process

We use [cargo-dist](https://github.com/axodotdev/cargo-dist) for automated releases.

1.  Bump the version in `Cargo.toml`.
2.  Push a git tag (e.g., `v0.1.1`).
3.  GitHub Actions will automatically build binaries for Linux, macOS, and Windows, create a GitHub Release, and update the Homebrew tap.

## 📜 License

By contributing, you agree that your contributions will be licensed under the Apache-2.0 License.
