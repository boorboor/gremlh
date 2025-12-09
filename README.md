# 👹 gremlh (Gremlin Hunter)

[![CI](https://github.com/boorboor/gremlh/actions/workflows/ci.yml/badge.svg)](https://github.com/boorboor/gremlh/actions/workflows/ci.yml)
[![Security Audit](https://github.com/boorboor/gremlh/actions/workflows/audit.yml/badge.svg)](https://github.com/boorboor/gremlh/actions/workflows/audit.yml)
[![Release](https://github.com/boorboor/gremlh/actions/workflows/release.yml/badge.svg)](https://github.com/boorboor/gremlh/actions/workflows/release.yml)
[![Crates.io](https://img.shields.io/crates/v/gremlh.svg)](https://crates.io/crates/gremlh)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**gremlh** is a blazing fast, multi-threaded CLI tool designed to hunt down and sanitize "gremlin" characters in your source code.

It identifies invisible characters, zero-width spaces, homoglyphs, and potential security risks (such as [Trojan Source](https://trojansource.codes/) attacks) that can cause compilation errors, syntax issues, or confusing bugs.

## 🚀 Features

* **🛡️ Security First:** Detects Bidi overrides (`\u202A` - `\u202E`) used in supply-chain attacks.
* **⚡ Parallel Scanning:** Built on the `ignore` crate (the same engine used by `ripgrep`) for maximum speed.
* **💾 Atomic Writes:** Fixes files safely. Changes are written to a temporary file and swapped only upon success.
* **🤖 Smart Detection:** Automatically skips binary files and respects `.gitignore` rules.
* **🐚 Shell Integration:** Comes with Man pages and completions for Bash, Zsh, and Fish.
* **✨ CI Ready:** Strict exit codes (1 for found gremlins, 0 for clean).

## 📦 Installation

### From Source (Recommended for Rust users)

```bash
cargo install gremlh
```

### Pre-built Binaries

Download pre-built binaries for Linux, macOS, and Windows from the [GitHub Releases](https://github.com/boorboor/gremlh/releases) page.

### Homebrew (macOS/Linux)

```bash
brew tap boorboor/gremlh
brew install gremlh
```

## 🛠️ Usage

### 1. Scan Mode (Default)

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

### 2. Fix Mode (`--write`)

Automatically cleans files in-place.

```bash
gremlh --write
```

**What gets fixed?**
* **Smart Quotes:** `“` → `"`
* **Non-Breaking Spaces:** `\u00A0` → `(Space)`
* **Zero-Width Characters:** Removed entirely.
* **BOM:** Byte Order Mark removed.
* **Bidi Characters:** Removed entirely.

### 3. Pipe Mode (STDIN)

Great for scripting or single-file processing.

```bash
cat dirty_file.txt | gremlh > clean_file.txt
```

## ⚙️ Command Line Options

| Flag | Short | Description |
|------|-------|-------------|
| `--write` | `-w` | Overwrite files in place with fixed content. |
| `--verbose` | `-v` | Show detailed processing info (files scanned, binary skips). |
| `--no-ignore` | | Ignore `.gitignore` and scan everything. |
| `--hidden` | | Search hidden files and directories (e.g., `.env`). |
| `--threads` | `-j` | Number of threads to use (defaults to CPU count). |

## 🔌 Integrations

### GitHub Actions (Native)

`gremlh` provides a native GitHub Action. Add this to your workflow to fail builds when gremlins are detected, or automatically fix them.

```yaml
steps:
  - uses: actions/checkout@v4

  - name: Gremlin Hunter
    uses: boorboor/gremlh@v0.1.0
    with:
      path: 'src/'          # Optional: defaults to '.'
      write: 'false'        # Optional: set to 'true' to fix files
      verbose: 'true'       # Optional: show scanned files
      # no-ignore: 'true'   # Optional: ignore .gitignore
      # hidden: 'true'      # Optional: scan hidden files
```

### Git Hooks (Pre-commit)

Add this to your `.pre-commit-config.yaml`. We provide two hooks: one for checking, and one for auto-fixing.

```yaml
-   repo: [https://github.com/boorboor/gremlh](https://github.com/boorboor/gremlh)
    rev: v0.1.0
    hooks:
    -   id: gremlh          # Fail on detection
    # OR
    -   id: gremlh-fix      # Auto-fix on commit
```

### Vim/Neovim

Auto-fix on save (add to your `init.vim` or `.vimrc`):

```vim
augroup GremlinKiller
    autocmd!
    autocmd BufWritePre *.rs,*.js,*.py silent! %!gremlh 2>/dev/null
augroup END
```

## 🔍 Detected Gremlins

| Category | Description | Example | Action (`--write`) |
|----------|-------------|---------|-------------------|
| **Security** | Trojan Source Bidi Overrides | `\u202A` | **Remove** |
| **Invisibles** | Zero Width Space, Joiners | `\u200B` | **Remove** |
| **Whitespace** | Non-breaking spaces | `\u00A0` | Replace with Space |
| **Quotes** | Smart/Curled Quotes | `“` `”` | Replace with ASCII `"` |
| **Homoglyphs** | Greek Question Mark | `;` | Replace with `;` |
| **Control** | Non-whitespace control chars | `\x07` | **Remove** |

## 💻 Development

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to set up the environment and add new gremlin definitions.

1.  **Install tools:** `cargo install prek`
2.  **Setup hooks:** `prek install`
3.  **Run checks:** `prek run --all-files`

## ⚠️ Known Limitations

* **Visual Columns:** Error reporting uses character indices, not visual columns (tabs are counted as 1 char).
* **Encodings:** Only UTF-8 is supported. UTF-16 files are typically treated as binary and skipped.

## 📄 License

This project is licensed under the [Apache-2.0 License](LICENSE).
