use std::io;
use std::io::{BufRead, BufReader, Result};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;

use crate::db::Server; // Use the Server struct from the db module

// Define events for server lifecycle
#[derive(Clone, Debug)]
pub enum ServerLifecycleEvent {
    Exited { name: String },
}

pub struct ServerHandle {
    pub child: Option<Child>,
    pub name: String,
    pub log_sender: Sender<String>, // Renamed for clarity
    pub server_event_sender: Sender<ServerLifecycleEvent>,
    pub running: bool,
}

impl ServerHandle {
    pub fn kill_process(&mut self) -> std::result::Result<(), String> {
        if let Some(ref mut child) = self.child {
            match child.kill() {
                Ok(_) => {
                    self.running = false;
                    // Send an exit event when killed
                    let _ = self.server_event_sender.send(ServerLifecycleEvent::Exited {
                        name: self.name.clone(),
                    });
                    Ok(())
                }
                Err(e) => Err(format!("Failed to kill server {}: {}", self.name, e)),
            }
        } else {
            // If there's no child process (e.g., dummy server or already stopped)
            self.running = false;
            // Send an exit event even for dummy servers or if no child process
            // Dummy Servers not exiting correctly
            let _ = self.server_event_sender.send(ServerLifecycleEvent::Exited {
                name: self.name.clone(),
            });

            Ok(())
        }
    }
}


fn build_command(server: &Server) -> Result<Command> {
    let (shell, shell_flag, change_dir_prefix) = match std::env::consts::OS {
        "windows" => ("cmd", "/C", "cd /d"),
        "macos" | "linux" => ("sh", "-c", "cd"),
        _ => {
            return Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported OS"));
        }
    };

    let cd_command = if std::env::consts::OS == "windows" {
        format!("{} {}", change_dir_prefix, server.path)
    } else {
        format!("{} '{}'", change_dir_prefix, server.path)
    };

    let full_command = format!(
        "{} && {} {}",
        cd_command,
        server.executable,
        server.args.join(" ")
    );

    let mut command_builder = Command::new(shell);
    command_builder
        .args([shell_flag, &full_command])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if std::env::consts::OS != "windows" {
        command_builder.current_dir(&server.path);
        let new_full_command = format!("{} {}", server.executable, server.args.join(" "));
        command_builder.args([shell_flag, &new_full_command]);
    }

    Ok(command_builder)
}

pub fn launch(
    server: &Server,
    log_sender: Sender<String>,
    server_event_sender: Sender<ServerLifecycleEvent>,
    dummy: bool,
) -> Result<ServerHandle> {
    if dummy {
        return dummy_launch(server, log_sender, server_event_sender);
    }

    let mut command = build_command(server)?;

    let mut child = command.spawn().map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("Failed to spawn server {}: {}", server.name, e),
        )
    })?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not capture stdout."))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not capture stderr."))?;

    let name_clone_stdout = server.name.clone();
    let log_sender_clone_stdout = log_sender.clone();
    let event_sender_clone_stdout = server_event_sender.clone();

    thread::spawn(move || {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(line_content) => {
                    if let Err(e) = log_sender_clone_stdout
                        .send(format!("[{}] {}", name_clone_stdout, line_content))
                    {
                        eprintln!("[{}] Error sending stdout log: {}", name_clone_stdout, e);
                    }
                }
                Err(e) => {
                    eprintln!("[{}] Error reading stdout line: {}", name_clone_stdout, e);
                    break;
                }
            }
        }
        let _ = event_sender_clone_stdout.send(ServerLifecycleEvent::Exited {
            name: name_clone_stdout,
        });
    });

    let name_clone_stderr = server.name.clone();
    let log_sender_clone_stderr = log_sender.clone();
    let event_sender_clone_stderr = server_event_sender.clone();

    thread::spawn(move || {
        let reader = std::io::BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(line_content) => {
                    if let Err(e) = log_sender_clone_stderr
                        .send(format!("[{}] [stderr] {}", name_clone_stderr, line_content))
                    {
                        eprintln!("[{}] Error sending stderr log: {}", name_clone_stderr, e);
                    }
                }
                Err(e) => {
                    eprintln!("[{}] Error reading stderr line: {}", name_clone_stderr, e);
                    break;
                }
            }
        }
        let _ = event_sender_clone_stderr.send(ServerLifecycleEvent::Exited {
            name: name_clone_stderr,
        });
    });

    Ok(ServerHandle {
        child: Some(child),
        name: server.name.clone(),
        log_sender,
        server_event_sender,
        running: true,
    })
}

// Dummy launch function updated to use the event sender
fn dummy_launch(
    server: &Server,
    log_sender: Sender<String>,
    server_event_sender: Sender<ServerLifecycleEvent>,
) -> Result<ServerHandle> {
    let name = server.name.clone();
    let log_sender_clone = log_sender.clone();
    let event_sender_clone = server_event_sender.clone();
    thread::spawn(move || {
        for i in 0..15 {
            if let Err(e) =
                log_sender_clone.send(format!("[{}] Dummy server running... {}", name, i))
            {
                eprintln!("[{}] Error sending dummy log: {}", name, e);
            }
            thread::sleep(std::time::Duration::from_secs(1));
        }
        let _ = event_sender_clone.send(ServerLifecycleEvent::Exited { name: name.clone() });
    });

    Ok(ServerHandle {
        child: None,
        name: server.name.clone(),
        log_sender,
        server_event_sender,
        running: true,
    })
}
