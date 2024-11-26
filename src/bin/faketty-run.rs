//! Minimal example of `faketty` without the `clap` dependency

use faketty_lib::run_command;

fn main() {
    let mut args = std::env::args();
    let argv0 = args.next().unwrap_or_default();
    let argv0 = match (env!("CARGO_BIN_NAME").trim(), argv0.trim()) {
        (x, _) | (_, x) if !x.is_empty() => x,
        _ => "faketty",
    };
    let args: Vec<_> = args
        .map(|x| std::ffi::CString::new(x.as_bytes()).unwrap())
        .collect();
    if args.is_empty() {
        eprintln!("Usage: {argv0} <program> <args...>");
        std::process::exit(1);
    };
    run_command(&args).unwrap();
}
