use std::fs::{self, File};
use std::io;
use std::process::Command;

#[cfg(test)]
fn test(program: &str) -> io::Result<()> {
    let tempdir = scratch::path("faketty");
    let stdout = tempdir.join("test-stdout");
    let stderr = tempdir.join("test-stderr");

    let status = Command::new(program)
        .arg("tests/test.sh")
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
    for program in [
        env!("CARGO_BIN_EXE_faketty"),
        env!("CARGO_BIN_EXE_faketty-run"),
    ] {
        test(program)?;
    }
    Ok(())
}
