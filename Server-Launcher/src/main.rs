mod tui;
mod servers;
mod db;

fn main() -> std::io::Result<()> {
    // get server states from local db

    if let Err(e) = tui::init_tui() {
        eprintln!("Application error: {}", e);
        // Optionally, perform any other cleanup before exiting
        std::process::exit(1);
    }
    Ok(())
}
