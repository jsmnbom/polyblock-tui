use backtrace::Backtrace;
use crossterm::{
    cursor::MoveTo,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io::{self, Write};
use std::iter;
use std::panic::{self, PanicInfo};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        Block, BorderType, Borders, Clear, Gauge, List, Paragraph, Row, Table, TableState, Text,
    },
    Terminal,
};

mod app;
mod help;
mod input;
mod ui;
mod util;

use app::{App, RouteId};
use util::{Event, Events, Key};

// https://github.com/Rigellute/spotify-tui/blob/master/src/main.rs#L91
fn panic_hook(info: &PanicInfo<'_>) {
    let location = info.location().unwrap();

    let msg = match info.payload().downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => match info.payload().downcast_ref::<String>() {
            Some(s) => &s[..],
            None => "Box<Any>",
        },
    };

    let stacktrace: String = format!("{:?}", Backtrace::new()).replace('\n', "\n\r");

    disable_raw_mode().unwrap();
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        Print(format!(
            "thread '<unnamed>' panicked at '{}', {}\n\r{}",
            msg, location, stacktrace
        )),
        DisableMouseCapture
    )
    .unwrap();
}

#[tokio::main]
async fn main() -> ::anyhow::Result<()> {
    panic::set_hook(Box::new(|info| {
        panic_hook(info);
    }));

    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = util::Events::new(100);

    let mut app = App::new();
    app.push_route_stack(RouteId::Home, false);

    loop {
        terminal.draw(|mut f| ui::draw_layout(&mut f, &mut app))?;

        match events.next()? {
            Event::Input(key) => {
                if key == Key::Ctrl('c') {
                    break;
                }

                input::handle(key, &mut app);
            }
            Event::Tick => {}
        }

        if app.should_quit {
            break;
        }
    }

    terminal.show_cursor()?;
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

    Ok(())
}
