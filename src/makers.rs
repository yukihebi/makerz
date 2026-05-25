use std::io;
use std::process::Command;

use crate::error::Error;

/// Name of the `makers` binary on PATH.
pub const MAKERS_BINARY: &str = "makers";

/// Build the argv that `makers` should be invoked with.
///
/// In the cli-skeleton scope this is the identity over the passthrough args.
/// Subsequent PRs prepend `--cwd <dir>` and `--env KEY=VALUE` pairs here.
pub fn build_args(passthrough: &[String]) -> Vec<String> {
    passthrough.to_vec()
}

/// Spawn `binary` with `args`, inherit stdio, and translate the result.
///
/// `binary` is parametric so callers (and tests) can swap it out;
/// the real entry point passes [`MAKERS_BINARY`].
pub fn spawn(binary: &str, args: &[String]) -> Result<(), Error> {
    let status = match Command::new(binary).args(args).status() {
        Ok(s) => s,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Err(Error::MakersNotFound),
        Err(err) => return Err(Error::MakersSpawn(err)),
    };
    if status.success() {
        Ok(())
    } else {
        Err(Error::MakersFailed {
            code: status.code(),
        })
    }
}

#[cfg(test)]
#[path = "makers_tests.rs"]
mod tests;
