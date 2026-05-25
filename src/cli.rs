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
    Scan::collect(args)?.classify()
}

/// Per-token accumulator. Filled by `collect`, consumed by `classify`.
#[derive(Default)]
struct Scan {
    help: bool,
    version: bool,
    init: bool,
    extend: Option<String>,
    extend_count: u32,
    passthrough: Vec<String>,
}

impl Scan {
    fn collect(args: Vec<String>) -> Result<Self, Error> {
        let mut scan = Scan::default();
        let mut iter = args.into_iter();
        while let Some(arg) = iter.next() {
            scan.consume(arg, &mut iter)?;
        }
        Ok(scan)
    }

    fn consume(&mut self, arg: String, rest: &mut std::vec::IntoIter<String>) -> Result<(), Error> {
        match arg.as_str() {
            "--help" => self.help = true,
            "--version" => self.version = true,
            "--init" => self.init = true,
            "--extend" => self.set_extend(extend_value_from_next(rest)?)?,
            s if s.starts_with("--extend=") => {
                self.set_extend(s["--extend=".len()..].to_string())?
            }
            _ => self.passthrough.push(arg),
        }
        Ok(())
    }

    fn set_extend(&mut self, value: String) -> Result<(), Error> {
        if value.is_empty() {
            return Err(Error::ArgParse(
                "--extend requires a non-empty path argument".into(),
            ));
        }
        self.extend_count += 1;
        self.extend = Some(value);
        Ok(())
    }

    fn classify(self) -> Result<Parsed, Error> {
        if self.help {
            return Ok(Parsed::Help);
        }
        if self.version {
            return Ok(Parsed::Version);
        }
        self.validate_extend()?;
        if self.init {
            ensure_no_passthrough(&self.passthrough)?;
            return Ok(Parsed::Init {
                extend: self.extend,
            });
        }
        Ok(Parsed::Passthrough {
            args: self.passthrough,
        })
    }

    fn validate_extend(&self) -> Result<(), Error> {
        if self.extend_count > 1 {
            return Err(Error::ArgParse(
                "--extend may be specified at most once".into(),
            ));
        }
        if self.extend.is_some() && !self.init {
            return Err(Error::ArgParse(
                "--extend requires --init (multi-parent or standalone extend is not supported)"
                    .into(),
            ));
        }
        Ok(())
    }
}

fn extend_value_from_next(rest: &mut std::vec::IntoIter<String>) -> Result<String, Error> {
    rest.next()
        .ok_or_else(|| Error::ArgParse("--extend requires a path argument".into()))
}

fn ensure_no_passthrough(passthrough: &[String]) -> Result<(), Error> {
    if !passthrough.is_empty() {
        return Err(Error::ArgParse(format!(
            "--init does not accept other arguments (got: {})",
            passthrough.join(" ")
        )));
    }
    Ok(())
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
