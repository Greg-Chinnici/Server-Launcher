use ratatui::style::Color;
use rusqlite::{params, Connection, Result};


#[derive(Debug, Clone)]
pub struct Server {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub executable: String,
    pub args: Vec<String>,
    pub autostart: bool,
    pub test_server: Option<bool>,
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
            test_server: row.get::<_, Option<bool>>(6)?
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
