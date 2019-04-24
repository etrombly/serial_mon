use byteorder::{ByteOrder, LE};
use serial::SerialPort;
use std::{
    io::{
        self, Write, prelude::*,
    },
    sync::mpsc,
    time::Duration,
    thread,
};
use termion::clear;
use termion::cursor::Goto;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::canvas::{Canvas, Points};
use tui::widgets::{Block, Borders, Gauge, Paragraph, Text, Widget};
use tui::Terminal;
use unicode_width::UnicodeWidthStr;

mod util;

use crate::util::event::{Config, Event, Events};

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// History of recorded messages
    messages: Vec<String>,
    port: serial::SystemPort,
}

impl App {
    fn read_serial(&mut self) -> io::Result<(f64, f32, f32)> {
        let mut buf: [u8; 13] = [0; 13];
        let mut data = [0; 12];

        self.port.read_exact(&mut buf[..])?;
        if cobs::decode(&buf, &mut data).is_ok() {
            let sleep = LE::read_u32(&data[0..4]);
            let x = LE::read_f32(&data[4..8]);
            let y = LE::read_f32(&data[8..12]);
            let cpu = if sleep > 64_000_000 {
                100.0
            } else {
                (64_000_000_f64 - f64::from(sleep)) / 64_000_000_f64 * 100_f64
            };
            return Ok((cpu, x, y));
        }
        Err(io::Error::new(io::ErrorKind::Other, "couldn't decode data"))
    }

    fn write_serial(&mut self, bytes: &[u8]) {
        while self.port.write(bytes).is_err() {
            thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            messages: Vec::new(),
            port: serial::open("/dev/ttyUSB0").unwrap(),
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
    app.port.reconfigure(&|settings| {
            settings.set_baud_rate(serial::Baud9600)?;
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            settings.set_flow_control(serial::FlowNone);
            Ok(())
    })?;
    app.port.set_timeout(Duration::from_millis(1))?;

    loop {
        terminal.draw(|mut f| {
            let mut cpu = 0.0_f64;
            let mut x = 0.0_f32;
            let mut y = 0.0_f32;
            if let Ok((t_cpu, t_x, t_y)) = app.read_serial() {
                cpu= t_cpu;
                x = t_x;
                y = t_y;
            }
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
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
                        Constraint::Percentage(10),
                    ]
                    .as_ref(),
                )
                .split(chunks[0]);
            Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("CPU"))
                .style(
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Black)
                        .modifier(Modifier::HIDDEN),
                )
                .percent(cpu as u16)
                .render(&mut f, status_chunks[0]);
            Canvas::default()
                .block(Block::default().title("Output").borders(Borders::ALL))
                .x_bounds([0.0, 100.0])
                .y_bounds([0.0, 100.0])
                .paint(|ctx| {
                    ctx.draw(&Points {
                        coords: &[(0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (3.0, 3.0)],
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
                    let line: String = app.input.drain(..).collect();
                    app.write_serial(line.as_bytes());
                    app.messages.push(line);
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
