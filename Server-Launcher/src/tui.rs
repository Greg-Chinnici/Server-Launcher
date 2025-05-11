

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*, style::palette::material::ORANGE, widgets::{Block, BorderType, Borders, Paragraph, Wrap}
};
use std::{io, time::Duration, error::Error};

use crate::db::Server;

struct App {
    counter: i32,
    // Placeholder for server logs
    logs: Vec<String>,
    available_servers :Vec<Server>,
    selected_server: usize,
}

impl App {
    fn new() -> App {
        App {
            counter: 0,
            logs: vec!["Log panel initialized.".to_string()],
            available_servers: vec![
                Server {
                    id: 1,
                    name: "Server 1".to_string(),
                    path: "/C".to_string(),
                    executable: "server.jar".to_string(),
                    args: vec!["arg1".to_string(), "arg2".to_string()],
                    autostart: false
                },
                Server {
                    id: 2,
                    name: "Server 2".to_string(),
                    path: "/C".to_string(),
                    executable: "server.jar".to_string(),
                    args: vec!["arg1".to_string(), "arg2".to_string()],
                    autostart: false
                },
                Server {
                    id: 3,
                    name: "Server 3".to_string(),
                    path: "/C".to_string(),
                    executable: "server.jar".to_string(),
                    args: vec!["arg1".to_string(), "arg2".to_string()],
                    autostart: false
                }
            ],
            selected_server: 0,
        }
    }


    fn on_tick(&mut self) {
        self.counter += 1;
        // Example: Add a new log entry periodically or when a server sends output
        // self.logs.push(format!("Tick: {}", self.counter));
        // Keep logs manageable
        if self.logs.len() > 20 {
            self.logs.remove(0);
        }
    }
}

pub fn init_tui() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error running TUI: {:?}", err);
        return Err(Box::new(err));
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::<B>(f, app))?;

        // Event handling with a timeout. 20fps
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('j') | KeyCode::Char('J') | KeyCode::Down => {
                            // Placeholder for moving down in server list
                            app.logs.push("Select Down".to_string());
                            app.selected_server = wrap_index(app.selected_server, app.available_servers.len()-1, -1);
                            app.logs.push(format!("new index is {}" , app.selected_server));
                        }
                        KeyCode::Char('k') | KeyCode::Char('K') | KeyCode::Up => {
                            // Placeholder for moving up in server list
                            app.logs.push("Select Up".to_string());
                            app.selected_server = wrap_index(app.selected_server , app.available_servers.len()-1,  1);
                            app.logs.push(format!("new index is {}" , app.selected_server));
                        }
                        KeyCode::Enter => {
                            // Placeholder for launching/modifying server
                            app.logs.push("Pressed Enter".to_string());
                            app.logs.push(format!("Selected server: {}", app.available_servers[app.selected_server].name));
                        }
                        KeyCode::Char('x') |  KeyCode::Char('X') => {
                            // Placeholder for killing/modifying server
                            app.logs.push("Pressed X".to_string());
                            app.logs.push(format!("Killing server: {}", app.available_servers[app.selected_server].name));
                        }
                        _ => {}
                    }
                }
                if key.kind == KeyEventKind::Repeat {
                    match key.code {
                        KeyCode::Enter => {
                                // Placeholder for launching/modifying server
                                app.logs.push("Pressed Enter as a Repeat".to_string());
                                app.logs.push(format!("Launching server: {}", app.available_servers[app.selected_server].name));
                            }
                        _ => {}
                    }
                }
            }

        }
        // Simple tick for now
        app.on_tick();
    }
}

fn ui<B: Backend>(f: &mut Frame<>, app: &App) {
    // Main vertical layout: one for app content, one for controls
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),       // Main content area takes the rest of the space
            Constraint::Length(3),    // Controls panel: 1 line for text, 2 for borders
        ].as_ref())
        .split(f.size());

    let content_area_chunk = main_chunks[0];
    let controls_chunk = main_chunks[1];

    // Horizontal layout for the content area (servers and logs)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(content_area_chunk); // Split the top part

    // Left Panel: Server List
    let left_panel_content = Paragraph::new(format!(
        "Server List\n\n{} Server 1\n{} Server 2\n{} Server 3\n\nCounter: {}",
        format!("○"),
        format!("○"),
        format!("○"),
        app.counter
    ))
    .block(Block::default().title("Servers").borders(Borders::ALL).border_style(Style::new().light_blue()))
    .wrap(Wrap { trim: true });
    f.render_widget(left_panel_content, content_chunks[0]);

    // Right Panel: Log Output
    let log_text: Vec<Line> = app.logs.iter().map(|log| Line::from(log.as_str())).collect();
    let right_panel_content = Paragraph::new(log_text)
        .block(Block::default().title("Log Stream").borders(Borders::ALL).border_style(Style::new().fg(Color::Rgb(255, 165, 0)))) // orange
        .wrap(Wrap { trim: true });
    f.render_widget(right_panel_content, content_chunks[1]);

    // Bottom Panel: Controls
    let controls_spans = Line::from(vec![
        Span::styled("Controls: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("(J or ↑) / (K or ↓) = Navigate | Enter = Select/Launch | Q = Quit"),
    ]);
    let controls_panel = Paragraph::new(controls_spans)
        .block(Block::default().title("Controls").borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(controls_panel, controls_chunk);
}


fn wrap_index(index: usize, max_index: usize, delta: isize) -> usize {
    let len = max_index + 1;
    let current_idx_signed = index as isize;
    let len_signed = len as isize;
    let new_idx_signed = current_idx_signed + delta;
    let result_signed = new_idx_signed.rem_euclid(len_signed);
    result_signed as usize
}
