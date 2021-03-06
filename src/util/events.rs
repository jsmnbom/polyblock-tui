/// Adapted from https://github.com/Rigellute/spotify-tui/tree/master/src/event
/// Only change is variable tick rate
use crossterm::event;
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

/// Represents an key.
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Key {
    /// Both Enter (or Return) and numpad Enter
    Enter,
    /// Tabulation key
    Tab,
    /// Backspace key
    Backspace,
    /// Escape key
    Esc,

    /// Left arrow
    Left,
    /// Right arrow
    Right,
    /// Up arrow
    Up,
    /// Down arrow
    Down,

    /// Insert key
    Ins,
    /// Delete key
    Delete,
    /// Home key
    Home,
    /// End key
    End,
    /// Page Up key
    PageUp,
    /// Page Down key
    PageDown,

    /// F0 key
    F0,
    /// F1 key
    F1,
    /// F2 key
    F2,
    /// F3 key
    F3,
    /// F4 key
    F4,
    /// F5 key
    F5,
    /// F6 key
    F6,
    /// F7 key
    F7,
    /// F8 key
    F8,
    /// F9 key
    F9,
    /// F10 key
    F10,
    /// F11 key
    F11,
    /// F12 key
    F12,
    Char(char),
    Ctrl(char),
    Alt(char),
    Unknown,
}

impl Key {
    /// Returns the function key corresponding to the given number
    ///
    /// 1 -> F1, etc...
    ///
    /// # Panics
    ///
    /// If `n == 0 || n > 12`
    pub fn from_f(n: u8) -> Key {
        match n {
            0 => Key::F0,
            1 => Key::F1,
            2 => Key::F2,
            3 => Key::F3,
            4 => Key::F4,
            5 => Key::F5,
            6 => Key::F6,
            7 => Key::F7,
            8 => Key::F8,
            9 => Key::F9,
            10 => Key::F10,
            11 => Key::F11,
            12 => Key::F12,
            _ => panic!("unknown function key: F{}", n),
        }
    }
}

impl From<event::KeyEvent> for Key {
    fn from(key_event: event::KeyEvent) -> Self {
        match key_event {
            event::KeyEvent {
                code: event::KeyCode::Esc,
                ..
            } => Key::Esc,
            event::KeyEvent {
                code: event::KeyCode::Backspace,
                ..
            } => Key::Backspace,
            event::KeyEvent {
                code: event::KeyCode::Left,
                ..
            } => Key::Left,
            event::KeyEvent {
                code: event::KeyCode::Right,
                ..
            } => Key::Right,
            event::KeyEvent {
                code: event::KeyCode::Up,
                ..
            } => Key::Up,
            event::KeyEvent {
                code: event::KeyCode::Down,
                ..
            } => Key::Down,
            event::KeyEvent {
                code: event::KeyCode::Home,
                ..
            } => Key::Home,
            event::KeyEvent {
                code: event::KeyCode::End,
                ..
            } => Key::End,
            event::KeyEvent {
                code: event::KeyCode::PageUp,
                ..
            } => Key::PageUp,
            event::KeyEvent {
                code: event::KeyCode::PageDown,
                ..
            } => Key::PageDown,
            event::KeyEvent {
                code: event::KeyCode::Delete,
                ..
            } => Key::Delete,
            event::KeyEvent {
                code: event::KeyCode::Insert,
                ..
            } => Key::Ins,
            event::KeyEvent {
                code: event::KeyCode::F(n),
                ..
            } => Key::from_f(n),
            event::KeyEvent {
                code: event::KeyCode::Enter,
                ..
            } => Key::Enter,
            event::KeyEvent {
                code: event::KeyCode::Tab,
                ..
            } => Key::Tab,

            // First check for char + modifier
            event::KeyEvent {
                code: event::KeyCode::Char(c),
                modifiers: event::KeyModifiers::ALT,
            } => Key::Alt(c),
            event::KeyEvent {
                code: event::KeyCode::Char(c),
                modifiers: event::KeyModifiers::CONTROL,
            } => Key::Ctrl(c),

            event::KeyEvent {
                code: event::KeyCode::Char(c),
                ..
            } => Key::Char(c),

            _ => Key::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// Configuration for event handling.
pub struct EventConfig {
    /// The key that is used to exit the application.
    pub exit_key: Key,
    /// The tick rate at which the application will sent an tick event.
    pub tick_rate_min: Duration,
    pub tick_rate_max: Duration,
}

impl Default for EventConfig {
    fn default() -> EventConfig {
        EventConfig {
            exit_key: Key::Ctrl('c'),
            tick_rate_min: Duration::from_millis(25),
            tick_rate_max: Duration::from_millis(250),
        }
    }
}

/// An occurred event.
pub enum Event<I> {
    /// An input event occurred.
    Input(I),
    /// An tick event occurred.
    Tick,
}

/// A small event handler that wrap crossterm input and tick event. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    // Need to be kept around to prevent disposing the sender side.
    _tx: mpsc::Sender<Event<Key>>,
}

impl Events {
    /// Constructs an new instance of `Events` with the default config.
    pub fn new(tick_rate_min: u64, tick_rate_max: u64) -> ::anyhow::Result<Events> {
        Events::with_config(EventConfig {
            tick_rate_min: Duration::from_millis(tick_rate_min),
            tick_rate_max: Duration::from_millis(tick_rate_max),
            ..Default::default()
        })
    }

    /// Constructs an new instance of `Events` from given config.
    pub fn with_config(config: EventConfig) -> ::anyhow::Result<Events> {
        let (tx, rx) = mpsc::channel();

        let event_tx = tx.clone();
        thread::Builder::new()
            .name("events".into())
            .spawn(move || {
                let mut last_tick = Instant::now();
                let mut key_sent = false;
                loop {
                    // poll for tick rate duration, if no event, sent tick event.
                    if event::poll(config.tick_rate_min).unwrap() {
                        if let event::Event::Key(key) = event::read().unwrap() {
                            // trace!("Got key: {:?}", key);
                            let key = Key::from(key);

                            event_tx.send(Event::Input(key)).unwrap();
                            key_sent = true;
                        }
                    }
                    if last_tick.elapsed() > config.tick_rate_min && key_sent {
                        // trace!("Over min tick rate - sending tick");
                        event_tx.send(Event::Tick).unwrap();
                        last_tick = Instant::now();
                        key_sent = false;
                    }
                    if last_tick.elapsed() > config.tick_rate_max {
                        // trace!("Over max tick rate - sending tick");
                        event_tx.send(Event::Tick).unwrap();
                        last_tick = Instant::now();
                    }
                }
            })?;

        Ok(Events { rx, _tx: tx })
    }

    /// Attempts to read an event.
    /// This function will block the current thread.
    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }
}
