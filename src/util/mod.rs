use data_encoding::HEXLOWER;
use sha1::{Digest, Sha1};
use std::{fs, path::Path};
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::Text,
};

mod events;
pub mod java;

pub use events::{Event, Events, Key};

pub fn wrap_dec(cur: usize, max: usize) -> usize {
    if cur == 0 {
        max - 1
    } else {
        cur - 1
    }
}

pub fn wrap_inc(cur: usize, max: usize) -> usize {
    if cur >= max - 1 {
        0
    } else {
        cur + 1
    }
}

pub fn centered_rect(height: u16, width: u16, r: Rect) -> Rect {
    Rect {
        height,
        width,
        x: (r.width - width) / 2,
        y: (r.height - height) / 2,
    }
}

pub fn centered_rect_percentage_dir(direction: Direction, percentage: u16, r: Rect) -> Rect {
    Layout::default()
        .direction(direction)
        .constraints(
            [
                Constraint::Percentage((100 - percentage) / 2),
                Constraint::Percentage(percentage),
                Constraint::Percentage((100 - percentage) / 2),
            ]
            .as_ref(),
        )
        .split(r)[1]
}

pub fn centered_rect_percentage(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let rect = centered_rect_percentage_dir(Direction::Vertical, percent_y, r);
    centered_rect_percentage_dir(Direction::Horizontal, percent_x, rect)
}

pub fn make_help<'a>(raw: Vec<(&'a str, &'a str)>) -> Vec<Text> {
    raw.into_iter()
        .map(|(key, text)| {
            vec![
                Text::styled(key, Style::default().modifier(Modifier::BOLD)),
                Text::raw(" "),
                Text::raw(text),
            ]
        })
        .collect::<Vec<_>>()
        .join(&Text::raw("   "))
}

//https://sts10.github.io/2019/06/06/is-all-equal-function.html
pub fn is_all_same<T: Eq>(slice: &[T]) -> bool {
    slice
        .get(0)
        .map(|first| slice.iter().all(|x| x == first))
        .unwrap_or(true)
}

pub fn sha1_file<P: AsRef<Path>>(path: P) -> ::anyhow::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha1::new();
    std::io::copy(&mut file, &mut hasher)?;
    let result = hasher.result();
    Ok(HEXLOWER.encode(result.as_ref()))
}
