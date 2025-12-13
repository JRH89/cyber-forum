// src/main.rs
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
    Terminal,
};
use anyhow::Result;

mod app;
// mod database; // Removed
// mod models; // Removed
mod api;

use app::{App, AppState, CurrentFocus};

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new();
    app.load_config();
    if !app.username_input.is_empty() && !app.password_input.is_empty() {
        let _ = app.login().await;
    }
    
    let res = run_app(&mut terminal, &mut app).await;
    
    restore_terminal(&mut terminal)?;
    
    if let Err(err) = res {
        println!("{:?}", err);
    }
    
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match app.state {
                AppState::Login => {
                    handle_login_keys(key, app).await?;
                }
                AppState::Forum => {
                    handle_forum_keys(key, app).await?;
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    match app.state {
        AppState::Login => draw_login_screen(f, app),
        AppState::Forum => draw_forum_ui(f, app),
    }
}

fn draw_login_screen(f: &mut ratatui::Frame, app: &App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(size);

    let title = Paragraph::new("TERNIMAL LOGIN")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let username_style = if app.focus == CurrentFocus::Username {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };
    let username = Paragraph::new(app.username_input.as_str())
        .style(username_style)
        .block(Block::default().borders(Borders::ALL).title("Username"));
    f.render_widget(username, chunks[1]);

    let password_style = if app.focus == CurrentFocus::Password {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };
    let password_display: String = app.password_input.chars().map(|_| '*').collect();
    let password = Paragraph::new(password_display)
        .style(password_style)
        .block(Block::default().borders(Borders::ALL).title("Password"));
    f.render_widget(password, chunks[2]);
}

fn draw_forum_ui(f: &mut ratatui::Frame, app: &App) {
    let size = f.size();
    
    // Check if we are in a modal mode (NewThread or Reply)
    if app.focus == CurrentFocus::NewThread {
        // Draw background (thread list) dimmed? Or just draw the modal over it.
        // Let's draw the modal.
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(5)
            .constraints([Constraint::Length(3), Constraint::Min(5), Constraint::Length(3)].as_ref())
            .split(size);
            
        let title_style = if app.new_thread_focus == CurrentFocus::Username { Style::default().fg(Color::Yellow) } else { Style::default() };
        let content_style = if app.new_thread_focus == CurrentFocus::ThreadList { Style::default().fg(Color::Yellow) } else { Style::default() }; // Reusing enums loosely here for sub-focus
        
        let title_input = Paragraph::new(app.new_thread_title.as_str())
            .block(Block::default().borders(Borders::ALL).title("Thread Title"))
            .style(title_style);
        f.render_widget(title_input, chunks[0]);
        
        let content_input = Paragraph::new(app.new_thread_content.as_str())
            .block(Block::default().borders(Borders::ALL).title("Content"))
            .style(content_style);
        f.render_widget(content_input, chunks[1]);
        
        let help = Paragraph::new("Tab: Switch Focus | Enter: Submit | Esc: Cancel")
            .style(Style::default().fg(Color::Gray));
        f.render_widget(help, chunks[2]);
        return;
    }
    
    if app.focus == CurrentFocus::Reply {
        let area = centered_rect(60, 40, size);
        let input = Paragraph::new(app.reply_content.as_str())
            .block(Block::default().borders(Borders::ALL).title("Reply Content"))
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(Clear, area); // Clear background
        f.render_widget(input, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(size);

    draw_thread_list(f, app, chunks[0]);
    draw_conversation(f, app, chunks[1]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ].as_ref())
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ].as_ref())
        .split(popup_layout[1])[1]
}

fn draw_thread_list(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .threads
        .iter()
        .map(|t| ListItem::new(t.title.clone()))
        .collect();

    let border_style = if app.focus == CurrentFocus::ThreadList {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Threads").border_style(border_style))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))
        .highlight_symbol("> ");

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.selected_thread));
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_conversation(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("Conversation");
    f.render_widget(block, area);

    let inner_area = area.inner(&ratatui::layout::Margin { vertical: 1, horizontal: 1 });
    
    if let Some(thread) = app.get_current_thread() {
        // Simple rendering: Title, Content, then comments
        let mut text = vec![
            Line::from(Span::styled(format!("Title: {}", thread.title), Style::default().add_modifier(Modifier::BOLD))),
            Line::from(Span::raw(format!("Author: {}", thread.author))),
            Line::from(Span::raw("")),
            Line::from(Span::raw(&thread.content)),
            Line::from(Span::raw("")),
            Line::from(Span::styled("--- Comments ---", Style::default().fg(Color::Gray))),
        ];
        
        for comment in &app.comments {
            text.push(Line::from(Span::styled(format!("{}:", comment.author), Style::default().fg(Color::Cyan))));
            text.push(Line::from(Span::raw(&comment.content)));
            text.push(Line::from(Span::raw("")));
        }
        
        let paragraph = Paragraph::new(text).wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(paragraph, inner_area);
    } else {
        let p = Paragraph::new("Select a thread to view").alignment(ratatui::layout::Alignment::Center);
        f.render_widget(p, inner_area);
    }
}

async fn handle_login_keys(key: crossterm::event::KeyEvent, app: &mut App) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Enter => {
            if !app.username_input.is_empty() && !app.password_input.is_empty() {
                app.login().await?;
            }
        }
        KeyCode::Tab => {
            app.focus = match app.focus {
                CurrentFocus::Username => CurrentFocus::Password,
                CurrentFocus::Password => CurrentFocus::Username,
                _ => CurrentFocus::Username,
            };
        }
        KeyCode::Char(c) => {
            match app.focus {
                CurrentFocus::Username => app.username_input.push(c),
                CurrentFocus::Password => app.password_input.push(c),
                _ => {}
            }
        }
        KeyCode::Backspace => {
            match app.focus {
                CurrentFocus::Username => {
                    app.username_input.pop();
                }
                CurrentFocus::Password => {
                    app.password_input.pop();
                }
                _ => {}
            }
        }
        KeyCode::Esc => app.should_quit = true,
        _ => {}
    }
    Ok(())
}

async fn handle_forum_keys(key: crossterm::event::KeyEvent, app: &mut App) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Up => {
            match app.focus {
                CurrentFocus::ThreadList => {
                    if app.selected_thread > 0 {
                        app.selected_thread -= 1;
                    }
                }
                _ => {}
            }
        }
        KeyCode::Down => {
            match app.focus {
                CurrentFocus::ThreadList => {
                    if app.selected_thread < app.threads.len().saturating_sub(1) {
                        app.selected_thread += 1;
                    }
                }
                _ => {}
            }
        }
        KeyCode::Left => app.focus = CurrentFocus::ThreadList,
        KeyCode::Right => {
            if !app.threads.is_empty() {
                app.focus = CurrentFocus::Conversation;
            }
        }
        KeyCode::Char('n') => {
            app.focus = CurrentFocus::NewThread;
            app.new_thread_title.clear();
            app.new_thread_content.clear();
            app.new_thread_focus = CurrentFocus::Username; // reuse as Title focus
        }
        KeyCode::Char('r') => {
            app.focus = CurrentFocus::Reply;
            app.reply_content.clear();
        }
        KeyCode::Enter => {
            match app.focus {
                CurrentFocus::ThreadList => {
                    app.open_thread(app.selected_thread).await?;
                }
                CurrentFocus::NewThread => {
                    let _ = app.create_thread(app.new_thread_title.clone(), app.new_thread_content.clone()).await;
                    app.load_threads().await?;
                    app.focus = CurrentFocus::ThreadList;
                }
                CurrentFocus::Reply => {
                    let _ = app.create_reply(app.reply_content.clone()).await;
                    app.focus = CurrentFocus::Conversation;
                }
                _ => {}
            }
        }
        KeyCode::Tab => {
            if app.focus == CurrentFocus::NewThread {
                app.new_thread_focus = match app.new_thread_focus {
                    CurrentFocus::Username => CurrentFocus::ThreadList, // Title -> Content
                    CurrentFocus::ThreadList => CurrentFocus::Username, // Content -> Title
                    _ => CurrentFocus::Username,
                };
            }
        }
        KeyCode::Esc => {
            if app.focus == CurrentFocus::NewThread || app.focus == CurrentFocus::Reply {
                app.focus = CurrentFocus::ThreadList;
            }
        }
        KeyCode::Char(c) => {
            if app.focus == CurrentFocus::NewThread {
                match app.new_thread_focus {
                    CurrentFocus::Username => app.new_thread_title.push(c),
                    CurrentFocus::ThreadList => app.new_thread_content.push(c),
                    _ => {}
                }
            } else if app.focus == CurrentFocus::Reply {
                app.reply_content.push(c);
            }
        }
        KeyCode::Backspace => {
            if app.focus == CurrentFocus::NewThread {
                match app.new_thread_focus {
                    CurrentFocus::Username => { app.new_thread_title.pop(); },
                    CurrentFocus::ThreadList => { app.new_thread_content.pop(); },
                    _ => {}
                }
            } else if app.focus == CurrentFocus::Reply {
                app.reply_content.pop();
            }
        }
        _ => {}
    }
    Ok(())
}
