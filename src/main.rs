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

mod util;

use crate::util::event::{Config, Event, Events};

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