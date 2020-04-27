use crossterm::{
    cursor, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{stdout, Write},
    panic,
    path::PathBuf,
    process,
    sync::{
        mpsc::{channel, Receiver},
        Arc,
    },
    thread,
};
use structopt::StructOpt;
use tokio::sync::RwLock;
use tui::{backend::CrosstermBackend, Terminal};

mod app;
mod forge;
mod input;
mod instance;
mod io;
mod minecraft;
mod mods;
mod paths;
mod routes;
mod ui;
mod util;

use app::App;
use instance::{Instance, Instances};
use io::{Io, IoEvent};
use paths::Paths;
use routes::Route;
use util::{Event, Events, Key};

pub fn cleanup_terminal() {
    disable_raw_mode().unwrap();
    execute!(stdout(), LeaveAlternateScreen, cursor::Show).unwrap();
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
        cleanup_terminal();
        better_panic::Settings::auto()
            .most_recent_first(false)
            .lineno_suffix(true)
            .create_panic_handler()(info);
        // Could be a panic from another thread - make sure we exit
        process::exit(1);
    }));

    env_logger::init();

    let opt = Opt::from_args();

    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new(25, 250)?;

    let (io_tx, io_rx) = channel::<IoEvent>();

    let app = Arc::new(RwLock::new(App::new(&opt, io_tx)?));
    let cloned_app = Arc::clone(&app);

    let reqwest_client = reqwest::Client::builder()
        .gzip(true)
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .build()?;
    thread::Builder::new().name("io".into()).spawn(move || {
        let mut io = Io::new(&app, reqwest_client);
        io_inner(io_rx, &mut io);
    })?;

    loop {
        {
            let mut app = cloned_app.write().await;
            app.hide_cursor = true;

            // Replicate start of terminal.draw so we can draw async
            terminal.autoresize()?;
            let mut frame = terminal.get_frame();
            ui::draw_layout(&mut frame, &mut app).await?;
            terminal.draw(|_| {})?;

            #[allow(irrefutable_let_patterns)]
            while let event = events.next()? {
                match event {
                    Event::Input(key) => {
                        if key == Key::Ctrl('c') {
                            app.should_quit = true;
                            break;
                        }

                        input::handle(key, &mut app);
                    }
                    Event::Tick => break,
                }
            }
        }
        {
            let app = cloned_app.read().await;

            if app.should_quit {
                break;
            }

            if app.hide_cursor {
                terminal.hide_cursor()?;
            } else {
                terminal.show_cursor()?;
            }
        }
    }

    cleanup_terminal();

    Ok(())
}

#[tokio::main]
async fn io_inner(io_rx: Receiver<IoEvent>, io: &mut Io) {
    while let Ok(io_event) = io_rx.recv() {
        match io.handle_io_event(io_event).await {
            Ok(_) => {}
            Err(e) => {
                panic!(format!("{:?}", e));
            }
        };
    }
}
