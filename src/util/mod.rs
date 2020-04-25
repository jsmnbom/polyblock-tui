use anyhow::Context;
use bytes::Buf;
use data_encoding::HEXLOWER;
use futures::stream::StreamExt;
use log::trace;
use sha1::{Digest, Sha1};
use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};
use tokio::io::AsyncWriteExt;
use tui::layout::{Constraint, Direction, Layout, Rect};

mod events;
pub mod java;
mod progress;

pub use events::{Event, Events, Key};
pub use progress::Progress;

pub fn wrap_dec(cur: usize, max: usize) -> usize {
    wrap_sub(cur, max, 1)
}

pub fn wrap_inc(cur: usize, max: usize) -> usize {
    wrap_add(cur, max, 1)
}

pub fn wrap_sub(cur: usize, max: usize, change: usize) -> usize {
    if cur == 0 {
        max - 1
    } else if cur < change {
        0
    } else {
        cur - change
    }
}

pub fn wrap_add(cur: usize, max: usize, change: usize) -> usize {
    if cur == max - 1 {
        0
    } else if cur > max - change {
        max - 1
    } else {
        cur + change
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

pub fn centered_rect_dir(direction: Direction, height_or_width: u16, r: Rect) -> Rect {
    match direction {
        Direction::Horizontal => Rect {
            height: r.height,
            width: height_or_width,
            x: (r.width - height_or_width) / 2,
            y: r.y,
        },
        Direction::Vertical => Rect {
            height: height_or_width,
            width: r.width,
            x: r.x,
            y: (r.height - height_or_width) / 2,
        },
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

pub async fn sha1_file_with_progress<P: AsRef<Path>>(
    pb: &Progress,
    path: P,
) -> ::anyhow::Result<String> {
    let file = fs::File::open(path)?;
    let mut hasher = Sha1::new();
    pb.set_length(file.metadata()?.len()).await;
    let mut reader = BufReader::new(file);

    loop {
        let length = {
            let buffer = reader.fill_buf()?;
            hasher.input(buffer);
            buffer.len()
        };
        pb.inc(length as u64).await;
        if length == 0 {
            break;
        }
        reader.consume(length);
    }

    let result = hasher.result();
    Ok(HEXLOWER.encode(result.as_ref()))
}

pub async fn download_file<P: Into<PathBuf>>(url: &str, path: P) -> ::anyhow::Result<()> {
    let path = path.into();
    fs::create_dir_all(path.parent().unwrap()).context("Couldn't create parent folder.")?;
    let mut file = tokio::fs::File::create(&path).await?;
    let response = reqwest::get(url).await?.error_for_status()?;
    let mut stream = response.bytes_stream();
    while let Some(v) = stream.next().await {
        file.write_all(&v?).await?;
    }
    file.flush().await?;

    Ok(())
}

pub async fn download_file_with_progress<P: Into<PathBuf>>(
    pb: &Progress,
    url: &str,
    path: P,
) -> ::anyhow::Result<()> {
    let path = path.into();
    fs::create_dir_all(path.parent().unwrap()).context("Couldn't create parent folder.")?;
    let mut file = tokio::fs::File::create(&path).await?;
    let response = reqwest::get(url).await?.error_for_status()?;
    let content_length = response.content_length();

    if let Some(content_length) = content_length {
        pb.set_length(content_length).await;
    }

    let mut stream = response.bytes_stream();
    while let Some(v) = stream.next().await {
        let mut v = v?;
        while v.has_remaining() {
            // Writes some prefix of the byte string, not necessarily
            // all of it.
            let written = file.write_buf(&mut v).await?;
            pb.inc(written as u64).await;
        }
    }
    file.flush().await?;
    Ok(())
}
