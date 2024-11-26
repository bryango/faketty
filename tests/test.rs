use std::fs::{self, File};
use std::io;
use std::process::Command;

#[cfg(test)]
fn test(binary_target: &str) -> io::Result<()> {
    let tempdir = scratch::path("faketty");
    let stdout = tempdir.join("test-stdout");
    let stderr = tempdir.join("test-stderr");

    let status = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--bin",
            binary_target,
            "--",
            "tests/test.sh",
        ])
        .stdout(File::create(&stdout)?)
        .stderr(File::create(&stderr)?)
        .status()?;

    assert_eq!(status.code(), Some(6));
    assert_eq!(fs::read(stdout)?, "stdout is tty\r\n".as_bytes());
    assert_eq!(fs::read(stderr)?, "stderr is tty\r\n".as_bytes());
    Ok(())
}

#[test]
fn test_all() -> io::Result<()> {
    #[allow(clippy::single_element_loop)]
    for program in [
        #[cfg(feature = "clap")]
        "faketty",
        "faketty-run",
    ] {
        test(program)?;
    }
    Ok(())
}
