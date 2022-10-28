use clap::{Parser, Subcommand};
use nix::libc;
use nix::unistd;
use std::env;
use std::error::Error;
use std::ffi::CString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install
    Install { name: String },
    /// Start a shell within the messy environment
    Shell,
}

fn main() -> Result<(), Box<dyn Error>> {
    use Commands::*;
    let cli = Cli::parse();
    match &cli.command {
        Install { name } => run_install_script(name),
        Shell => start_shell(),
    }
}

fn run_install_script(name: &str) -> Result<(), Box<dyn Error>> {
    use unistd::ForkResult::*;
    match unsafe { unistd::fork() }? {
        Child => {
            let root = get_messyenv_root()?;
            let mut scriptpath = root.clone();
            scriptpath.push("install-scripts");
            scriptpath.push(name);

            let mut workdir = root;
            workdir.push("workdir");
            workdir.push(name);
            fs::create_dir(&workdir)?;

            setmessyenv()?;
            env::set_current_dir(&workdir)?;
            let cmd = ["bash", scriptpath.as_os_str().to_str().unwrap()]
                .iter()
                .map(|s| CString::new(s.to_string()).unwrap())
                .collect::<Vec<_>>();
            let err = unistd::execvp(&cmd[0], &cmd).unwrap_err();
            println!("messyenv: {}", err.to_string());
            std::process::exit(1);
        }
        Parent { child: _ } => {
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

fn start_shell() -> Result<(), Box<dyn Error>> {
    use unistd::ForkResult;
    match unsafe { unistd::fork() }? {
        ForkResult::Child => {
            setmessyenv()?;
            exec_shell()
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

fn setmessyenv() -> Result<(), Box<dyn Error>> {
    unsafe {
        let mut dir = get_messyenv_root()?;
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
    Ok(())
}

fn exec_shell() -> Result<(), Box<dyn Error>> {
    let cmd = ["bash"]
        .iter()
        .map(|s| CString::new(s.to_string()).unwrap())
        .collect::<Vec<_>>();
    let err = unistd::execvp(&cmd[0], &cmd).unwrap_err();
    println!("messyenv: {}", err.to_string());
    std::process::exit(1);
}

fn get_messyenv_root() -> Result<PathBuf, Box<dyn Error>> {
    let mut dir = env::current_dir()?;
    dir.push(".messyenv");
    Ok(dir)
}
