use eyre::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Output;

pub fn ensure_success(out: &Output) -> Result<()> {
    if !out.status.success() {
        let err =
            std::str::from_utf8(&out.stderr).map_err(|e| eyre!("Failed to parse stderr: {}", e))?;
        bail!(
            "Command failed: {}\n{}",
            err,
            String::from_utf8_lossy(out.stderr.as_slice())
        )
    }
    Ok(())
}
#[allow(dead_code)]
pub fn ensure_output(out: &Output) -> Result<String> {
    ensure_success(out)?;
    let out = std::str::from_utf8(&out.stdout).map_err(|e| {
        eyre!(
            "Failed to parse stdout: {}\n{}",
            e,
            String::from_utf8_lossy(out.stdout.as_slice())
        )
    })?;

    Ok(out.to_owned())
}
