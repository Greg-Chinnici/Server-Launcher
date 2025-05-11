
mod tui;
mod servers;
mod db;



fn main() -> std::io::Result<()> {
    // get server states from local db
    let conn = db::connect_db("path_to_db");
    match conn{
        Ok(connection) => {
            let mut available_Servers = db::load_servers(&connection);
        },
        _ => {println!("Error geting servers")}
    }

    if let Err(e) = tui::init_tui() {
        eprintln!("Application error: {}", e);
        // Optionally, perform any other cleanup before exiting
        std::process::exit(1);
    }
    Ok(())
}
