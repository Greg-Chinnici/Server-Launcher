mod tui;
mod servers;

fn main() {
    println!("Hello, world!");
    println!("Current OS: {}", std::env::consts::OS);
    tui::init_tui();
}
