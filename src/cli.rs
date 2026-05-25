use crate::error::Error;

/// Result of parsing makerz's own CLI flags.
///
/// `--version`, `--help`, `--init`, and `--extend` are consumed by makerz.
/// Everything else is collected into `Passthrough::args` to be forwarded to `makers`.
#[derive(Debug, PartialEq, Eq)]
pub enum Parsed {
    /// `--version`: print makerz's version.
    Version,
    /// `--help`: print makerz's help.
    Help,
    /// `--init [--extend <path>]`: generate a new `Makefile.toml`.
    Init { extend: Option<String> },
    /// Forward `args` to `makers` as-is.
    Passthrough { args: Vec<String> },
}

/// Parse makerz's argv tail (program name already stripped).
pub fn parse(args: Vec<String>) -> Result<Parsed, Error> {
    let mut help = false;
    let mut version = false;
    let mut init = false;
    let mut extend: Option<String> = None;
    let mut extend_count: u32 = 0;
    let mut passthrough: Vec<String> = Vec::new();

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--help" => help = true,
            "--version" => version = true,
            "--init" => init = true,
            "--extend" => {
                extend_count += 1;
                let value = iter
                    .next()
                    .ok_or_else(|| Error::ArgParse("--extend requires a path argument".into()))?;
                if value.is_empty() {
                    return Err(Error::ArgParse(
                        "--extend requires a non-empty path argument".into(),
                    ));
                }
                extend = Some(value);
            }
            s if s.starts_with("--extend=") => {
                extend_count += 1;
                let value = &s["--extend=".len()..];
                if value.is_empty() {
                    return Err(Error::ArgParse(
                        "--extend requires a non-empty path argument".into(),
                    ));
                }
                extend = Some(value.to_string());
            }
            _ => passthrough.push(arg),
        }
    }

    if help {
        return Ok(Parsed::Help);
    }
    if version {
        return Ok(Parsed::Version);
    }

    if extend_count > 1 {
        return Err(Error::ArgParse(
            "--extend may be specified at most once".into(),
        ));
    }
    if extend.is_some() && !init {
        return Err(Error::ArgParse(
            "--extend requires --init (multi-parent or standalone extend is not supported)".into(),
        ));
    }

    if init {
        if !passthrough.is_empty() {
            return Err(Error::ArgParse(format!(
                "--init does not accept other arguments (got: {})",
                passthrough.join(" ")
            )));
        }
        return Ok(Parsed::Init { extend });
    }

    Ok(Parsed::Passthrough { args: passthrough })
}

/// Text printed for `makerz --version`.
pub fn version_text() -> String {
    format!(
        "makerz {}\n(for cargo-make's version, run `makers --version`)",
        env!("CARGO_PKG_VERSION"),
    )
}

/// Text printed for `makerz --help`.
pub fn help_text() -> &'static str {
    "\
makerz - a thin wrapper around cargo-make (`makers`)

USAGE:
    makerz [makers args...]            Forward args to `makers`
    makerz --init                      Generate a new Makefile.toml
    makerz --init --extend <path>      Generate Makefile.toml extending <path>
    makerz --version                   Print makerz's version
    makerz --help                      Print this help

For cargo-make's own usage, run `makers --help`.
"
}

#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
