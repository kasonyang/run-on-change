// Copyright 2018 Kason Yang
// Kason Yang (@kasonyang) <me@kason.site>

use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use globset::Glob;
use notify::{Watcher, RecursiveMode, Event};
use clap::Parser;
pub struct ChangeStatus {
    pub changed: bool,
    pub last_time: u128,
    pub last_file: String,
}

#[derive(Debug)]
pub struct AppError {
    message: String,
}

/// Run command on files changed
#[derive(Parser, Debug)]
pub struct CmdOption {

    /// Run command immediately after program start even though file is not changed
    #[arg(short, long)]
    immediate: bool,

    /// Do not print log message
    #[arg(short, long)]
    quiet: bool,

    /// Directory to watch
    #[arg(short, long, default_value_t = String::from("."))]
    directory: String,

    /// File pattern to watch, unix-style glob syntax
    pattern: String,

    /// Command to execute when files changed
    command: String,

    /// Command args
    command_args: Vec<String>,

}

fn app_error(message: String) -> AppError {
    AppError {
        message,
    }
}

fn main() {
    execute().unwrap_or_else(|e| {
        eprintln!("{}", e.message);
    });
}

fn execute() -> Result<(), AppError> {
    let option : CmdOption = CmdOption::parse();
    if option.immediate {
        run_command(&option);
    }
    let change_mutex_arc = Arc::new(Mutex::new(ChangeStatus {
        changed: false,
        last_time: 0,
        last_file: "".to_string(),
    }));
    let watch_path = Path::new(&option.directory).canonicalize()
        .map_err(|_e| app_error(format!("invalid watch directory:{}", &option.directory)))?;
    let watch_path_for_callback = watch_path.clone();
    let glob = Glob::new(&option.pattern)
        .map_err(|_e| app_error(format!("invalid pattern:{}", &option.pattern)))?
        .compile_matcher();
    let change_mutex = change_mutex_arc.clone();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        match res {
            Ok(event) => {
                event.paths.iter().for_each(|p| {
                    let relative_str = p.strip_prefix(&watch_path_for_callback).unwrap().to_str().unwrap();
                    if glob.is_match(relative_str) {
                        // println!("event: {:?} {:?}", &event.kind, relative_str);
                        let mut cs = change_mutex.lock().unwrap();
                        cs.changed = true;
                        cs.last_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                        cs.last_file = relative_str.to_string();
                    }
                });
            },
            Err(e) => eprintln!("unknown error: {:?}", e),
        }
    }).map_err(|e| app_error(format!("failed to create watcher:{}", e)))?;
    watcher.watch(&watch_path, RecursiveMode::Recursive)
        .map_err(|e| app_error(format!("failed to watch directory:{}", e)))?;
    let change_mutex = change_mutex_arc.clone();
    loop {
        thread::sleep(Duration::from_secs(1));
        let mut change_status = change_mutex.lock().unwrap();
        if !change_status.changed {
            continue;
        }
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        if now - change_status.last_time < 1000 {
            continue;
        }
        change_status.changed = false;
        change_status.last_time = 0;
        if !option.quiet {
            println!("change detected:{}", change_status.last_file);
        }
        run_command(&option)
    }
}

fn run_command(option: &CmdOption) {
    Command::new(&option.command)
        .args(&option.command_args)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}