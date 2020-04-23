use backtrace::Backtrace;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{stdout, Write},
    panic::{self, PanicInfo},
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver},
        Arc,
    },
};
use structopt::StructOpt;
use tokio::sync::Mutex;
use tui::{backend::CrosstermBackend, Terminal};

mod app;
mod input;
mod instance;
mod io;
mod minecraft;
mod mods;
mod paths;
mod ui;
mod util;
mod view;

use app::{App, RouteId};
use instance::{Instance, Instances};
use io::{Io, IoEvent};
use paths::Paths;
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
        stdout(),
        LeaveAlternateScreen,
        Print(format!(
            "thread '<unnamed>' panicked at '{}', {}\n\r{}",
            msg, location, stacktrace
        )),
        DisableMouseCapture
    )
    .unwrap();
}

#[derive(StructOpt, Debug)]
#[structopt(
    author,
    setting(structopt::clap::AppSettings::DontCollapseArgsInUsage),
    setting(structopt::clap::AppSettings::UnifiedHelpMessage),
    setting(structopt::clap::AppSettings::DisableHelpSubcommand)
)]
pub struct Opt {
    /// Increase verbosity (can be used multiple times)
    #[structopt(short, long = "verbose", parse(from_occurrences), global(true))]
    pub verbosity: usize,

    /// Overwrite default data directory
    ///
    /// This directory stores launcher profiles, libraries instances (worlds, mods, configs etc.).
    #[structopt(long = "data", parse(from_os_str), env = "POLYBLOCK_DATA")]
    pub data_directory: Option<PathBuf>,

    /// Overwrite default cache directory
    #[structopt(long = "cache", parse(from_os_str), env = "POLYBLOCK_CACHE")]
    pub cache_directory: Option<PathBuf>,

    /// Overwrite path to native minecraft launcher executable
    ///
    /// On windows this should be SOME_PATH/MinecraftLauncher.exe
    /// On other platforms it might vary but is usually SOME_PATH/minecraft-launcher
    #[structopt(long, parse(from_os_str), env = "POLYBLOCK_LAUNCHER")]
    pub launcher: Option<PathBuf>,

    /// Overwrite path to java home
    ///
    /// Note that java is only required for installation of forge
    #[structopt(long = "java", parse(from_os_str), env = "JAVA_HOME")]
    pub java_home: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> ::anyhow::Result<()> {
    panic::set_hook(Box::new(|info| {
        panic_hook(info);
    }));

    env_logger::init();

    let opt = Opt::from_args();

    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new(100);

    let (io_tx, io_rx) = channel::<IoEvent>();

    let app = Arc::new(Mutex::new(App::new(&opt, io_tx)?));

    let cloned_app = Arc::clone(&app);
    std::thread::spawn(move || {
        let mut io = Io::new(&app);
        io_inner(io_rx, &mut io);
    });

    loop {
        let mut app = cloned_app.lock().await;

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

        if app.hide_cursor {
            terminal.hide_cursor()?;
        } else {
            terminal.show_cursor()?;
        }
    }

    terminal.show_cursor()?;
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

    Ok(())
}

#[tokio::main]
async fn io_inner(io_rx: Receiver<IoEvent>, io: &mut Io) {
    while let Ok(io_event) = io_rx.recv() {
        match io.handle_io_event(io_event).await {
            Ok(_) => {}
            Err(e) => panic!(e),
        };
    }
}
