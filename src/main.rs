use anyhow::Result;
use std::sync::mpsc::{self, Receiver};
use std::{io, time::Duration};
use std::{thread, time::Instant};
use termion::raw::IntoRawMode;
use termion::{event::Key, input::TermRead};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols::Marker,
    widgets::{canvas::Canvas, Block, Borders, Clear},
    Terminal,
};

const MATO_HEAD: &str = "█";
const MATO_BODY: &str = "▓";
const RATE_MS: u64 = 100;

enum Event {
    Key(Key),
    Tick,
}

fn key_events() -> Receiver<Event> {
    let (tx, rx) = mpsc::channel();
    let tx2 = tx.clone();
    let tick_rate = Duration::from_millis(RATE_MS);

    let _key_handle = thread::spawn(move || {
        let stdin = io::stdin();
        for evt in stdin.keys() {
            if let Ok(key) = evt {
                if let Err(err) = tx.send(Event::Key(key)) {
                    eprintln!("{}", err);
                    return;
                }
            }
        }
    });
    let _tick_handle = {
        thread::spawn(move || loop {
            if tx2.send(Event::Tick).is_err() {
                break;
            }
            thread::sleep(tick_rate);
        })
    };

    rx
}

fn main() -> Result<()> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let events = key_events();
    let mut mato: Vec<(f64, f64)> = vec![(10.0, 10.0), (10.0, 11.0)];
    let mut width = 0.0;
    let mut height = 0.0;
    let mut last_key = Key::Up;
    let mut grow = false;
    let mut ticks = 0;

    loop {
        if mato.iter().skip(1).any(|xy| mato.first() == Some(xy)) {
            break;
        }
        terminal.draw(|f| {
            width = f.size().width as f64;
            height = f.size().height as f64;
            let title = format!("Mato - score: {}", ticks);
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.size());
            let canvas = Canvas::default()
                .block(
                    Block::default()
                        .title(title.as_str())
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::White)),
                )
                .marker(Marker::Block)
                .paint(|ctx| {
                    if let Some((x, y)) = mato.first() {
                        ctx.print(*x, *y, MATO_HEAD, Color::Yellow);
                        mato.iter().skip(1).for_each(|(x, y)| {
                            ctx.print(*x, *y, MATO_BODY, Color::Yellow);
                        })
                    }
                })
                .x_bounds([0.0, width])
                .y_bounds([0.0, height]);
            f.render_widget(canvas, chunks[0]);
        })?;

        match events.recv()? {
            Event::Key(key) => match key {
                Key::Char('q') => break,
                Key::Up | Key::Down | Key::Left | Key::Right => {
                    last_key = key;
                }
                _ => {}
            },
            Event::Tick => {
                ticks += 1;
                if ticks % 2 == 0 {
                    grow = true;
                } else {
                    grow = false;
                }
            }
        }

        match last_key {
            Key::Up => {
                if mato[0].1 < (height - 1.0) {
                    mato.insert(0, (mato[0].0, mato[0].1 + 1.0));
                } else {
                    mato.insert(0, (mato[0].0, 0.0));
                    mato[0].1 = 0.0;
                }
            }
            Key::Down => {
                if mato[0].1 >= 1.0 {
                    mato.insert(0, (mato[0].0, mato[0].1 - 1.0));
                } else {
                    mato.insert(0, (mato[0].0, height - 1.0));
                }
            }
            Key::Left => {
                if mato[0].0 >= 1.0 {
                    mato.insert(0, (mato[0].0 - 1.0, mato[0].1));
                } else {
                    mato.insert(0, (width - 1.0, mato[0].1));
                }
            }
            Key::Right => {
                if mato[0].0 < (width - 1.0) {
                    mato.insert(0, (mato[0].0 + 1.0, mato[0].1));
                } else {
                    mato.insert(0, (0.0, mato[0].1));
                }
            }
            _ => {}
        }
        if !grow {
            mato.truncate(mato.len() - 1);
        } else {
            grow = true;
        }
    }
    println!("Game over! Your score: {}", ticks);

    Ok(())
}
