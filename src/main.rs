use clap::{Parser, Subcommand};
use nix::libc;
use nix::unistd;
use std::env;
use std::error::Error;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a shell within the messy environment
    Shell,
}

fn main() -> Result<(), Box<dyn Error>> {
    use Commands::*;
    let cli = Cli::parse();
    match &cli.command {
        Shell => start_shell(),
    }
}

fn start_shell() -> Result<(), Box<dyn Error>> {
    use unistd::ForkResult;
    match unsafe { unistd::fork() }? {
        ForkResult::Child => exec_shell(),
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

fn exec_shell() -> Result<(), Box<dyn Error>> {
    unsafe {
        let mut dir = env::current_dir()?;
        dir.push(".messyenv");
        libc::setenv(
            CString::new("MESSYENVROOT")?.as_ptr(),
            CString::new(dir.clone().into_os_string().as_bytes())?.as_ptr(),
            1,
        );

        dir.push("local");
        libc::setenv(
            CString::new("MESSYENVLOCAL")?.as_ptr(),
            CString::new(dir.into_os_string().as_bytes())?.as_ptr(),
            1,
        );
    }
    let cmd = ["bash"]
        .iter()
        .map(|s| CString::new(s.to_string()).unwrap())
        .collect::<Vec<_>>();
    let err = unistd::execvp(&cmd[0], &cmd).unwrap_err();
    println!("messyenv: {}", err.to_string());
    std::process::exit(1);
}
