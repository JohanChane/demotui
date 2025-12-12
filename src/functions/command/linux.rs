use super::*;

pub fn correct_cap_for_tun() -> Result<String> {
    let binary_path = &crate::config::CONFIG.cfg_file.basic.clash_bin_path;

    exec("chmod", vec!["+x", binary_path])?;
    let mut path = std::env::var("PATH").unwrap_or_default();
    path.push_str(":/usr/sbin");
    let opt = Command::new("pkexec")
        .env("PATH", path)
        .args([
            "setcap",
            "'cap_net_admin,cap_net_bind_service=+ep'",
            binary_path,
        ])
        .output()?;

    Ok(stringify_output(opt))
}

pub(super) fn stringify_output(output: std::process::Output) -> String {
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    let result_str = format!(
        r#"{}
        Stdout:
        {}

        Stderr:
        {}
        "#,
        if output.status.success() {
            "OK".to_owned()
        } else {
            format!("Error({})", output.status)
        },
        stdout_str,
        stderr_str
    );

    result_str
}
