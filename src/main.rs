use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::layout::{Layout, Constraint, Direction, Alignment};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Widget, Block, Borders, Paragraph, Text, Gauge};
use termion::raw::IntoRawMode;
use termion::clear;
use termion::event::Key;
use termion::input::TermRead;

pub enum Event<I> {
    Input(I),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    input_handle: thread::JoinHandle<()>,
    tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: Key::Char('q'),
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    match evt {
                        Ok(key) => {
                            if let Err(_) = tx.send(Event::Input(key)) {
                                return;
                            }
                            if key == config.exit_key {
                                return;
                            }
                        }
                        Err(_) => {}
                    }
                }
            })
        };
        let tick_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let tx = tx.clone();
                loop {
                    tx.send(Event::Tick).unwrap();
                    thread::sleep(config.tick_rate);
                }
            })
        };
        Events {
            rx,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }
}

fn main() -> Result<(), io::Error> {
    println!("{}", clear::All);
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = Events::with_config(Config::default());
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
            .block(Block::default().title("Output").borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Left)
            .wrap(true)
            .render(&mut f, chunks[1]);
        let status_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(10),
                    Constraint::Percentage(90)
                ].as_ref()
            )
            .split(chunks[0]);
        Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("CPU"))
            .style(Style::default().fg(Color::White).bg(Color::Black).modifier(Modifier::ITALIC))
            .percent(20)
            .render(&mut f, status_chunks[0]);
    })
}