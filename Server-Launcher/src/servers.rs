
use std::collections::hash_map;
use std::error::Error;
use std::process::{Command, Child, Stdio};
use std::sync::mpsc::{Sender, channel};
use std::thread;
use std::io::{BufRead, BufReader, Result};
use std::io;

use crate::db::Server; // Use the Server struct from the db module

pub struct ServerHandle {
    pub child: Option<Child>,
    pub name: String,
    pub sender: Sender<String>,
    pub running: bool,
}


pub fn launch(server: &Server, log_sender: Sender<String>, dummy: bool) -> Result<ServerHandle>{

    if (dummy){
        return dummy_launch(server, log_sender);
    }

    let (shell, shell_flag, change_dir_prefix) = match std::env::consts::OS {
        "windows" => ("cmd", "/C", "cd /d"),
        "macos" | "linux" => ("sh", "-c", "cd"),
        _ => {
            return Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported OS"));
        }
    };

    // Construct the command to change directory and then execute the server.
    // For macos/linux, ensure the path is quoted if it contains spaces.
    let cd_command = if std::env::consts::OS == "windows" {
        format!("{} {}", change_dir_prefix, server.path)
    } else {
        format!("{} '{}'", change_dir_prefix, server.path)
    };

    let full_command = format!("{} && {} {}", cd_command, server.executable, server.args.join(" "));
    println!("Launching: {}", full_command); // Keep a simple print for now, color removed


    let mut command_builder = Command::new(shell);
    command_builder.args([shell_flag, &full_command])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Set the working directory directly on the Command builder if not on Windows,
    // as `cd` in `sh -c "cd ... && ..."` might not persist for the subsequent command
    // in all shell versions or scenarios.
    // For Windows, `cd /d` within `cmd /C` works as expected.
    if std::env::consts::OS != "windows" {
        command_builder.current_dir(&server.path);
        // Re-form full_command for non-Windows as cd is handled by current_dir
        let new_full_command = format!("{} {}", server.executable, server.args.join(" "));
        command_builder.args([shell_flag, &new_full_command]); // Reset args
        println!("Adjusted Launching (non-windows): sh -c \"{} {}\" in directory {}", server.executable, server.args.join(" "), server.path);
    }


    let mut child = command_builder.spawn().map_err(|e| {
        io::Error::new(e.kind(), format!("Failed to spawn server {}: {}", server.name, e))
    })?;

    let stdout = child.stdout.take().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not capture stdout."))?;
    let stderr = child.stderr.take().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not capture stderr."))?;
    let name_clone_stdout = server.name.clone();
    let sender_clone_stdout = log_sender.clone();

    // Spawn a thread to read stdout
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(line_content) => {
                    if let Err(e) = sender_clone_stdout.send(format!("[{}] {}", name_clone_stdout, line_content)) {
                        eprintln!("[{}] Error sending stdout log: {}", name_clone_stdout, e);
                    }
                }
                Err(e) => {
                    eprintln!("[{}] Error reading stdout line: {}", name_clone_stdout, e);
                }
            }
        }
    });

    let name_clone_stderr = server.name.clone();
    let sender_clone_stderr = log_sender.clone();
    // Same for stderr
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(line_content) => {
                    if let Err(e) = sender_clone_stderr.send(format!("[{}] [stderr] {}", name_clone_stderr, line_content)) {
                         eprintln!("[{}] Error sending stderr log: {}", name_clone_stderr, e);
                    }
                }
                Err(e) => {
                     eprintln!("[{}] Error reading stderr line: {}", name_clone_stderr, e);
                }
            }
        }
    });

    Ok(ServerHandle { child: Some(child), name: server.name.clone() , sender: log_sender ,running: true})
}

// Kill Server Function
fn dummy_launch(server: &Server, log_sender: Sender<String>) -> Result<ServerHandle>{
    let name = server.name.clone();
    let sender_clone = log_sender.clone();

    // Spawn a thread to simulate server launch
    thread::spawn(move || {
        for i in 0..5 {
            if let Err(e) = sender_clone.send(format!("[{}] Dummy server running... {}", name, i)) {
                let _ = sender_clone.send(format!("[{}] Error sending dummy log: {}", name, e));
            }
            thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    Ok(ServerHandle { child: None, name: server.name.clone(), sender: log_sender ,running: true})
}
