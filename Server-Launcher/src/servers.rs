use std::{error::Error, fmt::format};
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
struct Server {
    pub name: String,
    pub path: String,
    pub executable: String,
    pub args: Vec<String>,
    pub autostart: bool,
}

pub fn Launch(server: Server) {
    let (shell, shell_flag, change_dir) = match std::env::consts::OS {
        "windows" => ("cmd", "/C", format!("cd /d {}", server.path)),
        "macos" | "linux" => ("sh", "-c", format!("cd '{}' &&", server.path)),
        _ => {
            eprintln!("Unsupported OS");
            return;
        }
    };

    let full_command = format!("{} {} {}", change_dir, server.executable, server.args.join(" "));

    let status = Command::new(shell)
        .args([shell_flag, &full_command])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match status {
        Ok(_) => println!("Started: {}", server.name),
        Err(e) => eprintln!("Error starting {}: {}", server.name, e),
    }
}
