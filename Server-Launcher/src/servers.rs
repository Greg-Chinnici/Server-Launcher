use std::io::{self, Stdin};
use std::io::{BufRead, BufReader, Result, Read};
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
    pub log_sender: Sender<ServerMessage>, // Renamed for clarity
    pub server_event_sender: Sender<ServerLifecycleEvent>,
//    pub input: Stdin,
    pub running: bool,
}

pub struct ServerMessage{
    pub name: String,
    pub contents: String,
    pub is_err: bool,
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
    let os = std::env::consts::OS;
    let (shell, shell_flag, cd_prefix) = match os {
        "windows" => ("cmd", "/C", "cd /d"),
        "linux" | "macos" => ("sh", "-c", "cd"),
        _ => return Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported OS")),
    };

    let cd_command = format!("{} {}", cd_prefix, shell_escape(&server.path));
    let exec_command = format!("{} {}", server.executable, server.args.join(" "));
    let full_command = format!("{} && {}", cd_command, exec_command);

    let mut command = Command::new(shell);
    command
        .arg(shell_flag)
        .arg(full_command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    Ok(command)
}

fn shell_escape(path: &str) -> String {
    if std::env::consts::OS == "windows" {
        path.to_string()
    } else {
        format!("'{}'", path.replace('\'', "'\\''"))
    }
}

fn capture_output<R: Read + Send + 'static>(
    reader: R,
    name: String,
    is_stderr: bool,
    sender: Sender<ServerMessage>,
    event_sender: Sender<ServerLifecycleEvent>,
) {
    thread::spawn(move || {
        let reader = std::io::BufReader::new(reader);
        for line in reader.lines() {
            match line {
                Ok(line_content) => {
                    let prefix = if is_stderr { "[stderr] " } else { "" };
                    let msg = format!("{}{}", prefix, line_content);
                    if let Err(e) = sender.send(ServerMessage{name:name.clone(),  contents: msg , is_err: true}) {
                        eprintln!("[{}] Error sending log: {}", name, e);
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[{}] Error reading {} line: {}",
                        name,
                        if is_stderr { "stderr" } else { "stdout" },
                        e
                    );
                    break;
                }
            }
        }
        let _ = event_sender.send(ServerLifecycleEvent::Exited { name });
    });
}

pub fn launch(
    server: &Server,
    log_sender: Sender<ServerMessage>,
    server_event_sender: Sender<ServerLifecycleEvent>,
) -> Result<ServerHandle> {
    if server.test_server == Some(true) {
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
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not capture stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not capture stderr"))?;

    capture_output(
        stdout,
        server.name.clone(),
        false,
        log_sender.clone(),
        server_event_sender.clone(),
    );
    capture_output(
        stderr,
        server.name.clone(),
        true,
        log_sender.clone(),
        server_event_sender.clone(),
    );

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
    log_sender: Sender<ServerMessage>,
    server_event_sender: Sender<ServerLifecycleEvent>,
) -> Result<ServerHandle> {
    let name = server.name.clone();
    let log_sender_clone = log_sender.clone();
    let event_sender_clone = server_event_sender.clone();
    thread::spawn(move || {
        for i in 0..15 {
            if let Err(e) =
                log_sender_clone.send(ServerMessage{contents: format!("Dummy server running... {}", i), name: name.clone() , is_err: false})
            {
                eprintln!("[{}] Error sending dummy log: {}", name.clone(), e);
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
