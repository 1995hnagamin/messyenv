use clap::{Parser, Subcommand};
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use thiserror;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize messy environment
    Init,
    /// Install
    Install { name: String },
    /// Start a shell within the messy environment
    Shell,
}

#[derive(thiserror::Error, Debug)]
enum MessyError {
    #[error(".messyenv/ not found")]
    RootNotFound,
    #[error("child process error")]
    ChildProcessError { status: std::process::ExitStatus },
}

fn main() -> Result<(), Box<dyn Error>> {
    use Commands::*;
    let cli = Cli::parse();
    match &cli.command {
        Init => init_messyenv(),
        Install { name } => run_install_script(name),
        Shell => start_shell(),
    }
}

fn init_messyenv() -> Result<(), Box<dyn Error>> {
    use std::io::Write;
    let mut path = env::current_dir()?;
    path.push(".messyenv");
    fs::create_dir(&path)?;
    let root = path.clone();
    path.push("install-scripts");
    fs::create_dir(&path)?;
    path.pop();
    path.push("local");
    fs::create_dir(&path)?;
    path.pop();
    path.push("workdir");
    fs::create_dir(&path)?;
    path.pop();
    path.push("environment");
    let mut envfile = fs::File::create(path)?;
    envfile.write_all(include_bytes!("environment.in"))?;
    println!("Initialized messy environment ({})", root.display());
    Ok(())
}

fn run_install_script(name: &str) -> Result<(), Box<dyn Error>> {
    use std::process::Command;
    let root = get_messyenv_root()?;
    let mut scriptpath = root.clone();
    scriptpath.push("install-scripts");
    scriptpath.push(name);

    let mut workdir = root.clone();
    workdir.push("workdir");
    workdir.push(name);
    if workdir.as_path().exists() {
        let prompt = format!(
            "Path {} exists. Proceed anyway?",
            workdir.as_path().to_str().unwrap()
        );
        let proceed = ask_user_input(&prompt, false)?;
        if !proceed {
            std::process::exit(0);
        }
    } else {
        fs::create_dir(&workdir)?;
    }
    let mut melocal = root.clone();
    melocal.push("local");
    let mut child = Command::new("bash")
        .args([scriptpath.as_os_str()])
        .current_dir(workdir.as_os_str())
        .env("MESSYENVROOT", root.into_os_string())
        .env("MESSYENVLOCAL", melocal.into_os_string())
        .spawn()
        .expect("bash failed to start");
    let status = child.wait()?;
    if status.success() {
        println!("done.");
        Ok(())
    } else {
        Err(Box::new(MessyError::ChildProcessError { status }))
    }
}

fn start_shell() -> Result<(), Box<dyn Error>> {
    use std::ffi::OsStr;
    use std::process::Command;
    let meroot = get_messyenv_root()?;
    let mut melocal = meroot.clone();
    melocal.push("local");
    let mut ifilepath = meroot.clone();
    ifilepath.push("environment");
    let mut child = Command::new("bash")
        .args([OsStr::new("--init-file"), ifilepath.as_os_str()])
        .env("MESSYENVROOT", meroot.into_os_string())
        .env("MESSYENVLOCAL", melocal.into_os_string())
        .spawn()
        .expect("bash failed to start");
    let status = child.wait()?;
    if status.success() {
        println!("done.");
        Ok(())
    } else {
        Err(Box::new(MessyError::ChildProcessError { status }))
    }
}

fn ask_user_input(prompt: &str, default: bool) -> Result<bool, Box<dyn Error>> {
    use std::collections::HashMap;
    use std::io;
    use std::io::Write;

    let dict = [
        ("", default),
        ("yes", true),
        ("y", true),
        ("Y", true),
        ("no", false),
        ("n", false),
        ("N", false),
    ]
    .iter()
    .map(|(s, b)| (s.to_string(), *b))
    .collect::<HashMap<String, bool>>();
    loop {
        print!("{} [{}] > ", prompt, if default { "Y/n" } else { "y/N" });
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let answer = input.trim().to_string();
        match dict.get(&answer) {
            Some(val) => return Ok(*val),
            None => {}
        }
    }
}

fn get_messyenv_root() -> Result<PathBuf, Box<dyn Error>> {
    use std::path::Path;
    let mut dir = env::current_dir()?;
    dir.push(".messyenv");
    if Path::new(&dir).is_dir() {
        Ok(dir)
    } else {
        Err(Box::new(MessyError::RootNotFound))
    }
}
