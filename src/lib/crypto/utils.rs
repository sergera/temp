use eyre::*;
use reqwest::Request;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Output;

pub fn append_to_file(path: &str, data: impl AsRef<[u8]>) -> Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(data.as_ref())?;
    file.write(b"\n")?;
    Ok(())
}

pub fn request_to_curl(request: &Request) -> String {
    let mut curl = format!("curl -X {} {}", request.method(), request.url());
    for (key, value) in request.headers() {
        curl.push_str(&format!(" -H '{}: {}'", key, value.to_str().unwrap()));
    }
    if let Some(body) = request.body() {
        curl.push_str(&format!(
            " -d '{}'",
            std::str::from_utf8(body.as_bytes().unwrap()).unwrap()
        ));
    }
    curl
}

pub fn ensure_success(out: &Output) -> Result<()> {
    if !out.status.success() {
        let err =
            std::str::from_utf8(&out.stderr).map_err(|e| eyre!("Failed to parse stderr: {}", e))?;
        bail!(
            "openssl ecparam failed: {}\n{}",
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
