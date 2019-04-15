use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::canvas::{Canvas, Points};
use tui::layout::{Layout, Constraint, Direction, Alignment, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Widget, Block, Borders, Paragraph, Text, Gauge};
use termion::raw::IntoRawMode;
use termion::clear;
use termion::event::Key;
use termion::input::TermRead;
use termion::cursor::Goto;
use unicode_width::UnicodeWidthStr;

mod util;

use crate::util::event::{Config, Event, Events};

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// History of recorded messages
    messages: Vec<String>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            messages: Vec::new(),
        }
    }
}

fn main() -> Result<(), failure::Error> {
    println!("{}", clear::All);
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = Events::with_config(Config::default());
    // Create default app state
    let mut app = App::default();

    loop{
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(80),
                        Constraint::Percentage(20)
                    ].as_ref()
                )
                .split(f.size());
            Block::default()
                .title("Status")
                .borders(Borders::ALL)
                .render(&mut f, chunks[0]);
            Paragraph::new([Text::raw("test output")].iter())
                .block(Block::default().title("Serial").borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::Reset))
                .alignment(Alignment::Left)
                .wrap(true)
                .render(&mut f, chunks[1]);
            let status_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(80),
                        Constraint::Percentage(10)
                    ].as_ref()
                )
                .split(chunks[0]);
            Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("CPU"))
                .style(Style::default().fg(Color::White).bg(Color::Black).modifier(Modifier::HIDDEN))
                .percent(20)
                .render(&mut f, status_chunks[0]);
            Canvas::default()
                .block(Block::default().title("Output").borders(Borders::ALL))
                .x_bounds([0.0, 100.0])
                .y_bounds([0.0, 100.0])
                .paint(|ctx| {
                    ctx.draw(&Points {
                        coords: &[(0.0, 0.0), (1.0,1.0), (2.0, 2.0), (3.0, 3.0)],
                        color: Color::White,
                    });
                })
                .render(&mut f, status_chunks[1]);
            Paragraph::new([Text::raw(&app.input)].iter())
                .block(Block::default().title("Input").borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .alignment(Alignment::Left)
                .wrap(true)
                .render(&mut f, status_chunks[2]);
        })?;

        // Put the cursor back inside the input box
        write!(
            terminal.backend_mut(),
            "{}",
            Goto(4 + app.input.width() as u16, 32)
        )?;

        // Handle input
        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Char('\n') => {
                    app.messages.push(app.input.drain(..).collect());
                }
                Key::Char(c) => {
                    app.input.push(c);
                }
                Key::Backspace => {
                    app.input.pop();
                }
                _ => {}
            },
            _ => {}
        }
    }
    println!("{}", clear::All);
    Ok(())
}