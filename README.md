# gremlh (Gremlin Hunter)

[![CI](https://github.com/boorboor/gremlh/actions/workflows/ci.yml/badge.svg)](https://github.com/boorboor/gremlh/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/gremlh.svg)](https://crates.io/crates/gremlh)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**gremlh** is a blazing fast, multi-threaded CLI tool written in Rust designed to hunt down and sanitize "gremlin" characters in your source code.

It identifies invisible characters, homoglyphs, and potential security risks (such as Trojan Source attacks) that can cause compilation errors, syntax issues, or confusing bugs.

## 🚀 Features

- **🛡️ Security First:** Detects Bidi overrides (`\u202A` - `\u202E`) used in [Trojan Source](https://trojansource.codes/) attacks.
- **⚡ Parallel Scanning:** Built on the `ignore` crate (the same engine used by `ripgrep`) for maximum speed across large codebases.
- **💾 Atomic Writes:** Fixes files safely. Changes are written to a temporary file and swapped only upon successful completion.
- **🤖 Smart Detection:** Automatically skips binary files and respects `.gitignore` rules.
- **🔧 Zero Config:** Works out of the box with sensible defaults.
- **✨ CI Ready:** Returns strict exit codes (1 for found gremlins, 0 for clean) for pipeline integration.

## 📦 Installation

### From Source (Recommended)

```bash
cargo install --path .
````

### From Crates.io

```bash
cargo install gremlh
```

### Pre-built Binaries

Check the [Releases](https://www.google.com/search?q=https://github.com/boorboor/gremlh/releases) page for pre-built binaries for Linux, macOS, and Windows.

## 🛠️ Usage

### 1\. Scan Mode (Default)

Recursively scans the current directory. Prints issues to stderr and exits with code `1` if gremlins are found.

```bash
gremlh
# Or specify a path
gremlh ./src
```

**Example Output:**

```text
src/main.rs:10:45: found "​" (Zero Width Space)
src/legacy.js:2:15: found "“" (Smart Double Quote)
src/security.go:5:1: found "‮" (Bidirectional Text Override)
```

### 2\. Fix Mode (`--write`)

Automatically cleans files in-place.

```bash
gremlh --write
```

**What gets fixed?**

  - **Smart Quotes:** `“` → `"`
  - **Non-Breaking Spaces:** `\u00A0` → `(Space)`
  - **Zero-Width Characters:** Removed entirely.
  - **BOM:** Byte Order Mark removed.
  - **Bidi Characters:** Removed entirely.

### 3\. Pipe Mode (STDIN)

Great for scripting or single-file processing.

```bash
cat dirty_file.txt | gremlh > clean_file.txt
```

## ⚙️ Command Line Options

| Flag | Description |
|------|-------------|
| `--write`, `-w` | Overwrite files in place with fixed content. |
| `--verbose`, `-v` | Show detailed processing info (files scanned, binary skips). |
| `--no-ignore` | Ignore `.gitignore` and scan everything. |
| `--hidden` | Search hidden files and directories (e.g., `.env`). |
| `--threads`, `-j` | Number of threads to use (defaults to CPU count). |

## 🔍 Detected Gremlins

| Category | Description | Example | Action (`--write`) |
|----------|-------------|---------|-------------------|
| **Security** | Trojan Source Bidi Overrides | `\u202A` | **Remove** |
| **Invisibles** | Zero Width Space, Joiners | `\u200B` | **Remove** |
| **Whitespace** | Non-breaking spaces | `\u00A0` | Replace with Space |
| **Quotes** | Smart/Curled Quotes | `“` `”` | Replace with ASCII `"` |
| **Homoglyphs** | Greek Question Mark | `;` | Replace with `;` |
| **Control** | Non-whitespace control chars | `\x07` | **Remove** |

## ⚙️ Integrations

### Git Hooks (Pre-commit)

Add this to your `.pre-commit-config.yaml`:

```yaml
-   repo: local
    hooks:
    -   id: gremlh
        name: gremlh
        entry: gremlh
        language: system
        types: [text]
        exclude: \.svg$
```

### CI/CD (GitHub Actions)

Fail your build if invisible characters are introduced.

```yaml
steps:
  - uses: actions/checkout@v4
  - name: Install gremlh
    run: cargo install gremlh
  - name: Scan for gremlins
    run: gremlh .
```

### Vim/Neovim (Auto-fix on save)

```vim
augroup GremlinKiller
    autocmd!
    autocmd BufWritePre *.rs,*.js,*.py silent! %!gremlh 2>/dev/null
augroup END
```

## 💻 Development

1.  **Install tools:**
    ```bash
    cargo install prek
    ```
2.  **Setup hooks:**
    ```bash
    prek install
    ```
3.  **Run checks:**
    ```bash
    prek run --all-files
    ```

## ⚠️ Known Issues

  - **Visual Columns:** Error reporting uses character indices, not visual columns (tabs are counted as 1 char).
  - **Encodings:** Only UTF-8 is supported. UTF-16 files are typically treated as binary and skipped.

## 📄 License

This project is licensed under the [Apache-2.0 License](https://www.google.com/search?q=LICENSE).
