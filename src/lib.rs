#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

mod error;

pub use crate::error::Error;
use crate::error::Result;
use nix::pty::{self, ForkptyResult, Winsize};
use nix::sys::wait::{self, WaitStatus};
use nix::unistd::{self, Pid};
use std::convert::Infallible;
use std::ffi::CString;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use std::process;

const STDIN: BorrowedFd = unsafe { BorrowedFd::borrow_raw(0) };
const STDOUT: BorrowedFd = unsafe { BorrowedFd::borrow_raw(1) };
const STDERR: BorrowedFd = unsafe { BorrowedFd::borrow_raw(2) };

/// Runs a command in a pty, even if redirecting the output.
///
/// Internally the function calls [`exec(3)`], namely the child process will
/// replace the current (parent) process.
///
/// [`exec(3)`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/exec.html
///
#[allow(clippy::missing_errors_doc)]
pub fn run_command(args: &[CString]) -> Result<Infallible> {
    let new_stdin = STDIN.try_clone_to_owned()?;
    let new_stderr = STDERR.try_clone_to_owned()?;
    let pty1 = unsafe { crate::forkpty() }?;
    if let ForkptyResult::Parent { child, master } = pty1 {
        crate::copyfd(master.as_fd(), STDOUT);
        crate::copyexit(child);
    }
    let new_stdout = STDOUT.try_clone_to_owned()?;
    let pty2 = unsafe { crate::forkpty() }?;
    if let ForkptyResult::Parent { child, master } = pty2 {
        crate::copyfd(master.as_fd(), new_stderr.as_fd());
        crate::copyexit(child);
    }
    unistd::dup2(new_stdin.as_raw_fd(), STDIN.as_raw_fd())?;
    unistd::dup2(new_stdout.as_raw_fd(), STDOUT.as_raw_fd())?;
    crate::exec(args)
}

unsafe fn forkpty() -> Result<ForkptyResult> {
    let winsize = Winsize {
        ws_row: 24,
        ws_col: 80,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let termios = None;
    let result = unsafe { pty::forkpty(&winsize, termios) }?;
    Ok(result)
}

fn exec(args: &[CString]) -> Result<Infallible> {
    let args: Vec<_> = args.iter().map(CString::as_c_str).collect();
    Ok(unistd::execvp(args[0], &args)?)
}

fn copyfd(read: BorrowedFd, write: BorrowedFd) {
    const BUF: usize = 4096;
    let mut buf = [0; BUF];
    loop {
        match unistd::read(read.as_raw_fd(), &mut buf) {
            Ok(0) | Err(_) => return,
            Ok(n) => {
                let _ = write_all(write, &buf[..n]);
            }
        }
    }
}

fn write_all(fd: BorrowedFd, mut buf: &[u8]) -> Result<()> {
    while !buf.is_empty() {
        let n = unistd::write(fd, buf)?;
        buf = &buf[n..];
    }
    Ok(())
}

fn copyexit(child: Pid) -> ! {
    let flag = None;
    process::exit(match wait::waitpid(child, flag) {
        Ok(WaitStatus::Exited(_pid, code)) => code,
        _ => 0,
    });
}
