use nix::unistd;
use std::error::Error;
use std::ffi::CString;

fn main() -> Result<(), Box<dyn Error>> {
    use unistd::ForkResult;
    match unsafe { unistd::fork() }? {
        ForkResult::Child => {
            let cmd = ["bash"]
                .iter()
                .map(|s| CString::new(s.to_string()).unwrap())
                .collect::<Vec<_>>();
            let err = unistd::execvp(&cmd[0], &cmd).unwrap_err();
            println!("messyenv: {}", err.to_string());
            std::process::exit(1);
        }
        ForkResult::Parent { child: _ } => {
            use nix::sys::wait::WaitStatus::*;
            match nix::sys::wait::wait()? {
                Exited(_, _) => Ok(()),
                _ => {
                    panic!("unimpremented")
                }
            }
        }
    }
}
