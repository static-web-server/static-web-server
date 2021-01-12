use nix::errno::Errno;
use nix::libc::c_int;
use nix::sys::signal::{SIGCHLD, SIGINT, SIGTERM};
use nix::sys::wait::WaitStatus::{Exited, Signaled, StillAlive};
use nix::sys::wait::{waitpid, WaitPidFlag};
use nix::Error;

pub use signal::Signal;

/// It waits for an incoming Termination Signal like Ctrl+C (SIGINT), SIGTERM, etc
pub fn wait<F>(func: F)
where
    F: Fn(signal::Signal),
{
    let sig_trap = signal::trap::Trap::trap(&[SIGTERM, SIGINT, SIGCHLD]);
    for sig in sig_trap {
        match sig {
            SIGCHLD => {
                // Current std::process::Command ip does not have a way to find
                // process id, so we just wait until we have no children
                loop {
                    match waitpid(None, Some(WaitPidFlag::WNOHANG)) {
                        Ok(Exited(pid, status)) => {
                            println!("{} exited with status {}", pid, status);
                            continue;
                        }
                        Ok(Signaled(pid, sig, _)) => {
                            println!("{} killed by {}", pid, sig as c_int);
                            continue;
                        }
                        Ok(StillAlive) => break,
                        Ok(status) => {
                            println!("Temporary status {:?}", status);
                            continue;
                        }
                        Err(Error::Sys(Errno::ECHILD)) => return,
                        Err(e) => {
                            panic!("Error {:?}", e);
                        }
                    }
                }
            }
            sig => func(sig),
        }
    }
}

/// It casts a given `signal::Signal` to `i32`.
pub fn as_int(sig: signal::Signal) -> i32 {
    sig as c_int
}
