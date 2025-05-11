

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*, style::{palette::material::ORANGE, Modifier, Style}, terminal,
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap}
};
use std::io;
use std::time::Duration;
use std::collections::HashMap;
use std::error::Error;
use std::sync::mpsc::{Sender, Receiver, channel};

use crate::{db::Server, servers::ServerHandle};
use crate::servers;

struct App {
    counter: i32,
    // Placeholder for server logs
    logs: Vec<String>,
    available_servers :Vec<Server>,
    selected_server: usize,
    allocated_servers: HashMap<String, ServerHandle>,
    log_sender: Sender<String>,
    log_receiver: Receiver<String>,
}

impl App {
    fn new() -> App {
        let (log_sender, log_receiver) = channel();
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
            // TODO make this change when resize and drive the length of log vec
            allocated_servers: HashMap::new(),
            log_sender,
            log_receiver,
        }
    }


    fn on_tick(&mut self) {
        self.counter += 1;
        // Example: Add a new log entry periodically or when a server sends output
        // self.logs.push(format!("Tick: {}", self.counter));
        // Keep logs manageable - This is now handled in run_app
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
        let mut log_panel_frame_rect = Rect::default(); // To store the log panel's frame Rect

        terminal.draw(|f| {
            log_panel_frame_rect = ui::<B>(f, app); // ui now returns the log panel's frame Rect
        })?;

        // Event handling with a timeout. 20fps
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('j') | KeyCode::Char('J') | KeyCode::Down => {
                            // Placeholder for moving down in server list
                            app.selected_server = wrap_index(app.selected_server, app.available_servers.len()-1, 1);
                        }
                        KeyCode::Char('k') | KeyCode::Char('K') | KeyCode::Up => {
                            // Placeholder for moving up in server list
                            app.selected_server = wrap_index(app.selected_server , app.available_servers.len()-1,  -1);
                        }
                        KeyCode::Enter => {
                            // Placeholder for launching/modifying server
                            app.logs.push(format!("Selected server: {}", app.available_servers[app.selected_server].name));

                            match servers::launch(&app.available_servers[app.selected_server] , app.log_sender.clone() , true)
                            {
                                Ok(handle) => {
                                    app.allocated_servers.insert(app.available_servers[app.selected_server].name.clone(), handle);
                                    app.logs.push(format!("Server {} launched successfully.", app.available_servers[app.selected_server].name));
                                }
                                Err(e) => {
                                    app.logs.push(format!("Failed to launch server {}: {}", app.available_servers[app.selected_server].name, e));
                                }
                            }
                        }
                        KeyCode::Char('x') |  KeyCode::Char('X') => {
                            // Placeholder for killing/modifying server
                            app.logs.push(format!("Killing server: {}", app.available_servers[app.selected_server].name));
                        }
                        KeyCode::Char('c') |  KeyCode::Char('C') => {
                            app.logs.clear();
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
        // Check for new log messages
        while let Ok(log_message) = app.log_receiver.try_recv() {
            app.logs.push(log_message);
        }

        // Trim logs to fit the panel, ensuring we keep the latest entries
        let displayable_log_lines = log_panel_frame_rect.height.saturating_sub(2).max(0) as usize; // -2 for borders, ensure non-negative
        if app.logs.len() > displayable_log_lines {
            app.logs.drain(0..(app.logs.len() - displayable_log_lines));
        }

        // Simple tick for now
        app.on_tick();

    }
}

fn ui<B: Backend>(f: &mut Frame<>, app: &App) -> Rect { // Return the Rect of the log panel frame
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
    let server_items: Vec<ListItem> = app.available_servers
        .iter()
        .enumerate()
        .map(|(i, server)| {
            if i == app.selected_server {
                let line = Line::from(Span::styled(
                    format!("> {}", server.name),
                    Style::default().add_modifier(Modifier::UNDERLINED),
                ));
                ListItem::new(line)
            } else {
                ListItem::new(Line::from(format!("  {}", server.name))).style(Style::new().red())
            }
        })
        .collect();

        let server_list = List::new(server_items)
            .block(Block::default().title("Servers").borders(Borders::ALL).border_style(Style::new().light_blue()))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        f.render_widget(server_list, content_chunks[0]);

    // Right Panel: Log Output
    let log_panel_frame_rect = content_chunks[1]; // The Rect for the entire log panel widget (frame included)

    let log_text: Vec<Line> = app.logs.iter().map(|log| Line::from(log.as_str())).collect();

    // Calculate the inner height of the log panel for scrolling content (area inside borders)
    let inner_log_area_height = log_panel_frame_rect.height.saturating_sub(2).max(0); // Subtract 2 for top/bottom borders

    // Calculate scroll offset to show the latest logs.
    // If app.logs.len() is less than or equal to inner_log_area_height, scroll_offset_y will be 0.
    // Otherwise, it scrolls to ensure the last `inner_log_area_height` lines are visible at the bottom.
    let scroll_offset_y = (app.logs.len() as u16).saturating_sub(inner_log_area_height);

    let right_panel_content = Paragraph::new(log_text)
        .block(Block::default().title("Log Stream").borders(Borders::ALL).border_style(Style::new().fg(Color::Rgb(255, 165, 0)))) // orange
        .wrap(Wrap { trim: true })
        .scroll((scroll_offset_y, 0)); // Add scroll to show the bottom of the logs
    f.render_widget(right_panel_content, log_panel_frame_rect);

    // Bottom Panel: Controls
    let controls_spans = Line::from(vec![
        Span::styled("Controls: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("(J or Down) / (K or Up) = Navigate | Enter = Select/Launch | Q = Quit"),
    ]);
    let controls_panel = Paragraph::new(controls_spans)
        .block(Block::default().title("Controls").borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(controls_panel, controls_chunk);

    content_chunks[1] // Return the Rect of the log panel's frame
}


fn wrap_index(index: usize, max_index: usize, delta: isize) -> usize {

    let len = max_index + 1;
    let current_idx_signed = index as isize;
    let len_signed = len as isize;
    let new_idx_signed = current_idx_signed + delta;
    let result_signed = new_idx_signed.rem_euclid(len_signed);
    result_signed as usize
}
