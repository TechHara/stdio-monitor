use clap::Parser;
use std::fs::File;
use std::io::Result;
use std::io::{copy, stderr, stdin, stdout, Read, Write};
use std::process::{exit, Command, Stdio};

/// monitor stdin/stdout/stderr
#[derive(Parser, Debug)]
#[command(override_usage = "stdio-monitor [OPTIONS] -- <COMMAND>...")]
struct Args {
    /// Path to log stdin traffic; default to stderr
    #[arg(long)]
    stdin: Option<String>,

    /// Path to log stdout traffic; default to stderr
    #[arg(long)]
    stdout: Option<String>,

    /// Path to log stderr traffic; default to stderr
    #[arg(long)]
    stderr: Option<String>,

    /// command to execute the program with arguments...
    #[arg(required = true)]
    command: Vec<String>,
}

fn parse_option(path: Option<String>) -> Result<Box<dyn Write>> {
    if path.is_none() {
        Ok(Box::new(stderr()))
    } else {
        let file = File::create(path.unwrap())?;
        Ok(Box::new(file))
    }
}

fn tee(mut read: impl Read, mut w1: impl Write, mut w2: impl Write) -> Result<usize> {
    const BUF_SIZE: usize = 1024;
    let mut total = 0;
    let mut buf = [0; BUF_SIZE];
    loop {
        let n = read.read(&mut buf)?;
        if n == 0 {
            break;
        }
        total += n;
        w1.write_all(&buf[..n])?;
        w1.flush()?;
        w2.write_all(&buf[..n])?;
        w2.flush()?;
    }

    Ok(total)
}

fn main() -> Result<()> {
    let args = Args::parse();
    eprintln!("Launching: {}", &args.command.join(" "));
    let mut child = Command::new(&args.command[0])
        .args(&args.command[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let child_stdin = child.stdin.take().expect("Failed to get child's stdin");
    let mut child_stderr = child.stderr.take().expect("Failed to get child's stderr");
    let child_stdout = child.stdout.take().expect("Failed to get child's stdout");

    // read from stdin and forward to the child stdin while logging
    let _ = std::thread::spawn(move || {
        let stdin_logger = parse_option(args.stdin).expect("Failed to open stdin logger");
        tee(stdin(), stdin_logger, child_stdin).expect("Failed to tee stdin");
    });

    // read from child stdout and forward to stdout while logging
    let stdout_thread = std::thread::spawn(move || {
        let stdout_logger = parse_option(args.stdout).expect("Failed to open stdout logger");
        tee(child_stdout, stdout_logger, stdout()).expect("Failed to tee stdout");
    });

    // log child stderr
    let stderr_thread = std::thread::spawn(move || {
        let mut stderr_logger = parse_option(args.stderr).expect("Failed to open stderr logger");
        copy(&mut child_stderr, &mut stderr_logger).expect("Failed to forward stderr");
    });

    // Wait for the program and last two threads to finish
    let status = child.wait()?;
    stderr_thread.join().expect("Failed to join stderr thread");
    stdout_thread.join().expect("Failed to join stdout thread");

    // stdin thread won't join, so need to exit explicitly
    if !status.success() {
        eprintln!("program exited with {:?}", status);
        exit(-1);
    } else {
        exit(0);
    }
}
