use std::{
    io::{Error, ErrorKind},
    os::unix::process::CommandExt,
    process::Command,
};

pub fn check_app_native(app_name: &str) -> Result<String, Error> {
    let check_cmd = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };

    let output = Command::new(check_cmd).arg(app_name).output()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(path)
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            format!(
                "Apps '{}' not found. Please install to use this app (https://mpv.io/installation/)",
                app_name
            ),
        ))
    }
}

pub fn play_url(url: &str) -> Result<(), Error> {
    // call mpv
    let mut child = Command::new("mpv")
        .args([url, "--no-video", "--really-quiet"])
        .process_group(0)
        .spawn()
        .expect("failed to start mpv with url");

    let status = child.wait()?;

    if !status.success() {
        eprintln!("Mpv exit with error")
    }

    Ok(())
}
