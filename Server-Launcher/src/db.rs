use ratatui::style::Color;
use rusqlite::{params, Connection, Result};


#[derive(Debug, Clone)]
pub struct Server {
    pub id: i32,
    pub name: String, // Alias shown in the menu
    pub path: String, // Absoltute path to the items directory on disk
    pub executable: String, // Shell env. (python3, sh ...etc)
    pub args: Vec<String>,
    pub autostart: bool, // If True will laucnh the server when the program starts
    pub test_server: Option<bool>, // If true it uses a Dummy Server Thread
    pub display_color: ratatui::style::Color
}

impl Server {
    fn default() -> Server {
        Server { id: -1, name: "".to_string(), path: "~/Users/student/bin".to_string(), executable: "script.sh".to_string(), args: vec!["".to_string()], autostart: false, test_server: Some(false), display_color: Color::White }
    }

}

pub fn connect_db(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS servers (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            executable TEXT NOT NULL,
            args TEXT NOT NULL,
            autostart INTEGER NOT NULL
        )",
        [],
    )?;

    Ok(conn)
}

pub fn load_servers(conn: &Connection) -> Result<Vec<Server>> {
    let mut stmt =
        conn.prepare("SELECT id, name, path, executable, args, autostart FROM servers")?;
    let rows = stmt.query_map([], |row| {
        Ok(Server {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            executable: row.get(3)?,
            args: row
                .get::<_, String>(4)?
                .split_whitespace()
                .map(String::from)
                .collect(),
            autostart: row.get::<_, i32>(5)? != 0,
            test_server: row.get::<_, Option<bool>>(6)?,
            display_color: row.get::<ratatui::style::Color>(7)
        })
    })?;

    Ok(rows.filter_map(Result::ok).collect())
}

pub fn update_server_args(conn: &Connection, id: i32, new_args: &[String]) -> Result<()> {
    let joined = new_args.join(" ");
    conn.execute(
        "UPDATE servers SET args = ?1 WHERE id = ?2",
        params![joined, id],
    )?;
    Ok(())
}

pub fn insert_server(conn: &Connection, server: &Server) -> Result<()> {
    let joined = server.args.join(" ");
    conn.execute(
        "INSERT INTO servers (name, path, executable, args, autostart) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            server.name,
            server.path,
            server.executable,
            joined,
            server.autostart as i32,
        ],
    )?;
    Ok(())
}
