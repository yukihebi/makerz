# makerz

A thin wrapper around [cargo-make](https://github.com/sagiegurari/cargo-make) (`makers`) that improves working-directory handling and lets tasks reference the user's original invocation directory.

> **Status:** early development. The discovery and `caller` directive are usable today. `file` / `inherit` directives and `makerz --init` are planned.

## Why

`cargo-make` is powerful but inconvenient in two ways that makerz fixes:

1. **You can only run it where `Makefile.toml` lives.** From any subdirectory, tasks resolve paths relative to the wrong place. `cargo make` / `makers` does not search upward.
2. **The original invocation directory is lost.** Once cargo-make changes to the Makefile's directory, scripts cannot tell where the user actually typed the command — even though that is often what task arguments are relative to.

makerz solves (1) by walking up to find `Makefile.toml` and telling makers to use it. It solves (2) with a one-line comment directive that captures the original cwd into an environment variable of your choice.

## Install

makerz is not yet on crates.io.

```sh
cargo install --path .
```

`makers` (= cargo-make) must be on `PATH`:

```sh
cargo install cargo-make
```

## Usage

```sh
makerz [<makers args>...]
```

makerz consumes only its own flags (`--version`, `--help`; in future also `--init` / `--extend`). Everything else — including cargo-make's own flags like `--quiet`, task names, and trailing task arguments — is passed through to `makers` unchanged.

### Makefile discovery

makerz walks upward from the current directory and runs `makers` with `--cwd <found-dir>` pointing at the first `Makefile.toml` it finds.

```
my-project/
├── Makefile.toml          ← makerz finds this
└── src/foo/               ← `makerz build` works from here
```

If no `Makefile.toml` exists between the current directory and the filesystem root, makerz exits non-zero with an error.

### The `caller` directive

Once makers receives `--cwd`, its working directory becomes the Makefile's directory, and the user's original invocation directory is gone. The `caller` directive captures it into an env variable.

```toml
# Makefile.toml
[env]
# @makerz = "caller"
CALLER_DIR = "."

[tasks.run]
cwd = "${SOME_OTHER_DIR}"
script = "my-tool ${CALLER_DIR}/${1}"
```

- When invoked through `makerz`, `${CALLER_DIR}` is the absolute path of the directory from which you ran `makerz`.
- When invoked through plain `makers`, the fallback (`"."`) is used. This keeps the Makefile usable both ways.
- The variable name is yours — `CALLER_DIR` is just a convention.

The directive is a TOML comment on the line *immediately preceding* the env key it binds to (blank lines and other comments in between are allowed; a section header or end-of-file is not). At most one `caller` directive is permitted per Makefile.

### `--version` and `--help`

```sh
makerz --version    # prints makerz's version
makerz --help       # prints makerz's usage summary
```

For cargo-make's own version or help, run `makers --version` / `makers --help`.

## Limitations and roadmap

Implemented:

- Upward `Makefile.toml` discovery + `--cwd` injection
- `caller` directive (single-Makefile scope)

Planned:

- `file` directive — capture *this* Makefile's directory as an absolute path
- `inherit` directive — propagate a parent Makefile's `file` value through `extend` chains, so tasks survive being extended from elsewhere
- `makerz --init` / `makerz --init --extend <path>` — scaffold a new `Makefile.toml` with the helper env block prefilled
- Validation of `extend` chains (existence, cycle detection, inherit continuity)

Out of scope for now: multi-parent `extend = [...]`, `extend` with `relative`/`optional` attributes, and the `CARGO_MAKE_EXTEND_WORKSPACE_MAKEFILE` workspace mechanism.

Windows is not officially supported yet: CI only runs Linux and we have not verified behavior on Windows. The current code has no platform-specific paths and may well work, but expect rough edges once we add path-embedding features (`file` / `inherit` / `--init --extend`).

## Development

```sh
cargo test                          # unit tests + CI-safe integration tests
cargo test --features with-makers   # also run end-to-end tests (requires `makers`)
cargo fmt
cargo clippy --all-targets -- -D warnings
```
