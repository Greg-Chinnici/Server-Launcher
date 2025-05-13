use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use std::collections::{vec_deque, HashMap, VecDeque};
use std::error::Error;
use std::io;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

use crate::servers::{self, ServerLifecycleEvent};
use crate::{db::Server, servers::ServerHandle};

struct App {
    counter: i32,
    // Placeholder for server logs
    logs: VecDeque<String>,
    available_servers: Vec<Server>,
    selected_server: usize,
    allocated_servers: HashMap<String, ServerHandle>,
    // direct log data
    log_sender: Sender<String>,
    log_receiver: Receiver<String>,
    // server open / close
    server_event_sender: Sender<ServerLifecycleEvent>,
    server_event_receiver: Receiver<ServerLifecycleEvent>,
}

impl App {
    fn new() -> App {
        let (log_sender, log_receiver) = channel();
        let (server_event_sender, server_event_receiver) = channel();
        App {
            counter: 0,
            logs: VecDeque::from(["Log panel initialized.".to_string()]),
            available_servers: vec![
                Server {
                    id: 1,
                    name: "Server 1".to_string(),
                    path: "/C".to_string(),
                    executable: "server.jar".to_string(),
                    args: vec!["arg1".to_string(), "arg2".to_string()],
                    autostart: false,
                    test_server: Some(true)
                },
                Server {
                    id: 2,
                    name: "Timer 1".to_string(),
                    path: "/Users/student/Projects/Server-Launcher/Server-Launcher/".to_string(),
                    executable: "python3".to_string(),
                    args: vec!["-u".to_string() , "timer.py".to_string(), "8".to_string()], // needs the -u to run python in unbuffered mode
                    autostart: true,
                    test_server: Some(false)
                },
                Server {
                    id: 3,
                    name: "Timer 2".to_string(),
                    path: "/Users/student/Projects/Server-Launcher/Server-Launcher/".to_string(),
                    executable: "python3".to_string(),
                    args: vec!["-u".to_string() , "timer.py".to_string(), "8".to_string()], // needs the -u to run python in unbuffered mode
                    autostart: true,
                    test_server: Some(false)
                },
                Server {
                    id: 4,
                    name: "Timer 3 (custom)".to_string(),
                    path: "/Users/student/Projects/Server-Launcher/Server-Launcher/".to_string(),
                    executable: "python3".to_string(),
                    args: vec!["-u".to_string() , "timer.py".to_string(), "8".to_string() , "custom python message".to_string()], // needs the -u to run python in unbuffered mode
                    autostart: true,
                    test_server: Some(false)
                },
                Server {
                    id: 5,
                    name: "Timer 4".to_string(),
                    path: "/Users/student/Projects/Server-Launcher/Server-Launcher/".to_string(),
                    executable: "python3".to_string(),
                    args: vec!["-u".to_string() , "timer.py".to_string(), "8".to_string()], // needs the -u to run python in unbuffered mode
                    autostart: true,
                    test_server: Some(false)
                },
                Server {
                    id: 6,
                    name: "Minecraft Server".to_string(),
                    path: "/C".to_string(),
                    executable: "server.jar".to_string(),
                    args: vec!["arg1".to_string(), "arg2".to_string()],
                    autostart: false,
                    test_server: Some(true)

                },
            ],
            selected_server: 0,
            allocated_servers: HashMap::new(),
            log_sender,
            log_receiver,
            server_event_sender,
            server_event_receiver,
        }
    }

    fn on_tick(&mut self) {
        self.counter += 1;

        let mut server_names_to_remove = Vec::new();

        for (name, handle) in self.allocated_servers.iter_mut() {
            if handle.running {
                // Only check/update servers that are supposed to be running
                if let Some(ref mut child) = handle.child {
                    match child.try_wait() {
                        Ok(Some(_status)) => {
                            // Process has exited
                            self.logs
                                .push_back(format!("Server {} process has exited.", name));
                            handle.running = false; // Mark as not running
                        }
                        Ok(None) => { /* Process is still running */ }
                        Err(e) => {
                            self.logs.push_back(format!(
                                "Error checking status for server {}: {}. Marking as not running.",
                                name, e
                            ));
                            handle.running = false; // Mark as not running on error
                        }
                    }
                } else {
                    // For servers without a child process (e.g., dummy servers),
                    // their `running` flag is primarily managed by lifecycle events
                    // (like Exited) or explicit `kill_process` calls.
                    // `try_wait` is not applicable here.
                }
            }

            // If, after checks or an explicit kill, the handle is marked as not running, schedule for removal.
            if !handle.running {
                server_names_to_remove.push(name.clone());
            }
        }

        for name in server_names_to_remove {
            self.allocated_servers.remove(&name);
        }
    }
}

pub fn init_tui() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

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

fn run_app<backend: Backend>(terminal: &mut Terminal<backend>, app: &mut App) -> io::Result<()> {

    loop {
        let mut log_panel_frame_rect = Rect::default(); // To store the log panel's frame Rect

        terminal.draw(|f| {
            log_panel_frame_rect = ui::<backend>(f, app); // ui now returns the log panel's frame Rect
        })?;

        // Event handling with a timeout. 1000 / 50 => 20fps
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('j') | KeyCode::Char('J') | KeyCode::Down => {
                            app.selected_server =
                                wrap_index(app.selected_server, app.available_servers.len() - 1, 1);
                        }
                        KeyCode::Char('k') | KeyCode::Char('K') | KeyCode::Up => {
                            app.selected_server = wrap_index(
                                app.selected_server,
                                app.available_servers.len() - 1,
                                -1,
                            );
                        }
                        KeyCode::Enter => {
                            if app
                                .allocated_servers
                                .contains_key(&app.available_servers[app.selected_server].name)
                            {
                                continue;
                            }

                            match servers::launch(
                                &app.available_servers[app.selected_server],
                                app.log_sender.clone(),
                                app.server_event_sender.clone(),
                            ) {
                                Ok(handle) => {
                                    app.allocated_servers.insert(
                                        app.available_servers[app.selected_server].name.clone(),
                                        handle,
                                    );
                                    app.logs.push_back(format!(
                                        "Server {} launched successfully.",
                                        app.available_servers[app.selected_server].name
                                    ));
                                }
                                Err(e) => {
                                    app.logs.push_back(format!(
                                        "Failed to launch server {}: {}",
                                        app.available_servers[app.selected_server].name, e
                                    ));
                                }
                            }
                        }
                        KeyCode::Char('x') | KeyCode::Char('X') => {
                            if !app.available_servers.is_empty() {
                                let server_name_to_kill =
                                    app.available_servers[app.selected_server].name.clone();
                                if let Some(handle) =
                                    app.allocated_servers.get_mut(&server_name_to_kill)
                                {
                                    match handle.kill_process() {
                                        Ok(_) => {
                                            app.logs.push_back(format!("Attempting to kill server: {}. It will be removed from the list if successful.", server_name_to_kill));
                                        }
                                        Err(e) => {
                                            app.logs.push_back(format!(
                                                "Failed to kill server {}: {}",
                                                server_name_to_kill, e
                                            ));
                                        }
                                    }
                                } else {
                                    app.logs.push_back(format!(
                                        "Server {} is not currently running or allocated.",
                                        server_name_to_kill
                                    ));
                                }
                            }
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            app.logs.clear();
                        }
                        _ => {}
                    }
                }
            }
        }

        while let Ok(log_message) = app.log_receiver.try_recv() {
            app.logs.push_back(log_message);
        }

        while let Ok(event) = app.server_event_receiver.try_recv() {
            match event {
                ServerLifecycleEvent::Exited { name } => {
                    app.logs.push_back(format!("Server {} has exited.", name));
                    if let Some(handle) = app.allocated_servers.get_mut(&name) {
                        handle.running = false;
                    }
                }
            }
        }

        // Trim logs to fit the panel, ensuring we keep the latest entries
        let displayable_log_lines = log_panel_frame_rect.height.saturating_sub(2).max(0) as usize;
        if app.logs.len() > displayable_log_lines {
            app.logs.drain(0..(app.logs.len() - displayable_log_lines));
        }

        app.on_tick();
    }
}

fn ui<backend: Backend>(frame: &mut Frame, app: &App) -> Rect {
    // Return the Rect of the log panel frame
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(0),    // Main content area takes the rest of the space
                Constraint::Length(4), // Controls panel: 2 lines for text, 2 for borders
            ]
            .as_ref(),
        )
        .split(frame.size());

    let content_area_chunk = main_chunks[0];
    let controls_chunk = main_chunks[1];

    // Horizontal layout for the content area (servers and logs)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(content_area_chunk); // Split the top part

    // Left Panel: Server List
    let server_items: Vec<ListItem> = app
        .available_servers
        .iter()
        .enumerate()
        .map(|(i, server)| {
            let line = Line::from(Span::styled(
                if i == app.selected_server {
                    format!("> {}", server.name)
                } else {
                    format!("{}", server.name)
                },
                style_builder(i, server.clone(), app),
            ));
            ListItem::new(line)
        })
        .collect();

    let server_list = List::new(server_items)
        .block(
            Block::default()
                .title("Servers")
                .borders(Borders::ALL)
                .border_style(Style::new().light_blue()),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    frame.render_widget(server_list, content_chunks[0]);

    // Right Panel: Log Output
    let log_panel_frame_rect = content_chunks[1]; // The Rect for the entire log panel widget (frame included)

    let log_text: Vec<Line> = app
        .logs
        .iter()
        .map(|log| Line::from(log.as_str()))
        .collect();

    // Calculate the inner height of the log panel for scrolling content (area inside borders)
    let inner_log_area_height = log_panel_frame_rect.height.saturating_sub(2).max(0);

    let scroll_offset_y = (app.logs.len() as u16).saturating_sub(inner_log_area_height);

    let right_panel_content = Paragraph::new(log_text)
        .block(
            Block::default()
                .title("Log Stream")
                .borders(Borders::ALL)
                .border_style(Style::new().fg(Color::Rgb(255, 165, 0))),
        ) // orange
        .wrap(Wrap { trim: true })
        .scroll((scroll_offset_y, 0)); // Add scroll to show the bottom of the logs
    frame.render_widget(right_panel_content, log_panel_frame_rect);

    // Bottom Panel: Controls
    let controls_line1 = Line::from(vec![Span::raw(
        "(J/Down, K/Up) Navigate Servers | (Enter) Launch/Select",
    )]);
    let controls_line2 = Line::from(vec![Span::raw(
        "(X) Kill Server | (C) Clear Logs | (Q/Esc) Quit",
    )]);

    let controls_text = vec![controls_line1, controls_line2];

    let controls_panel = Paragraph::new(controls_text)
        .block(Block::default().title("Controls").borders(Borders::ALL))
        .alignment(Alignment::Center);
    frame.render_widget(controls_panel, controls_chunk);

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

fn style_builder(index: usize, server: Server, app: &App) -> Style {
    let mut style = Style::new();

    if app.allocated_servers.contains_key(&server.name) {
        style = style.fg(Color::Green);
    } else {
        style = style.fg(Color::Red);
    }

    if index == app.selected_server {
        style = style.patch(
            Style::default()
                .add_modifier(Modifier::UNDERLINED)
                .add_modifier(Modifier::BOLD),
        );
    }

    style
}
