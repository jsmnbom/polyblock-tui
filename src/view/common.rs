use crossterm::{cursor::MoveTo, execute};
use log::debug;
use std::{
    io::{self, Write},
    iter,
};
use tui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, Gauge, Paragraph, Row, Table, TableState, Text},
};

use unicode_width::UnicodeWidthStr;

use crate::{
    ui::{RenderState, UiFrame},
    util,
};

pub fn draw_button_dialog(
    f: &mut UiFrame<'_>,
    chunk: Rect,
    height: u16,
    text: &str,
    buttons: Vec<&str>,
    selected_button_i: usize,
) -> RenderState {
    let rect = util::centered_rect_percentage_dir(Direction::Horizontal, 60, chunk);
    let rect = util::centered_rect_dir(Direction::Vertical, height, rect);

    let block_widget = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain);
    f.render_widget(Clear, rect);
    f.render_widget(block_widget, rect);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(height - 6),
                Constraint::Length(1),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(rect);

    f.render_widget(
        Paragraph::new([Text::raw(text)].iter())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow))
            .wrap(true),
        layout[0],
    );

    let buttons_widths: Vec<u16> = buttons.iter().map(|s| s.len() as u16).collect();
    let buttons_width: u16 =
        buttons_widths.iter().sum::<u16>() + 3u16 * (buttons_widths.len() - 1) as u16;
    let mut constraints = Vec::new();
    constraints.push(Constraint::Length((layout[2].width - buttons_width) / 2));
    for button_width in buttons_widths {
        constraints.push(Constraint::Length(button_width));
        constraints.push(Constraint::Length(3));
    }
    constraints.pop();
    constraints.push(Constraint::Length((layout[2].width - buttons_width) / 2));

    let button_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(layout[2]);

    for (i, button) in buttons.into_iter().enumerate() {
        f.render_widget(
            Paragraph::new(vec![Text::raw(button)].iter()).style(if selected_button_i == i {
                Style::default().fg(Color::Blue).modifier(Modifier::BOLD)
            } else {
                Style::default().modifier(Modifier::DIM)
            }),
            button_layout[(i * 2) + 1],
        );
    }

    RenderState::default()
}

pub fn draw_input_dialog(
    f: &mut UiFrame<'_>,
    chunk: Rect,
    title: &str,
    entered_text: &str,
    error: Option<&str>,
) -> RenderState {
    let rect = util::centered_rect_percentage_dir(Direction::Horizontal, 30, chunk);
    let rect = util::centered_rect_dir(
        Direction::Vertical,
        if error.is_some() { 4 } else { 3 },
        rect,
    );
    f.render_widget(Clear, rect);

    let mut text = vec![Text::raw(entered_text)];
    if let Some(error) = error {
        text.push(Text::raw("\n\r"));
        text.push(Text::styled(error, Style::default().fg(Color::Red)));
    }

    f.render_widget(
        Paragraph::new(text.iter())
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Plain)
                    .title(title),
            ),
        rect,
    );

    execute!(
        io::stdout(),
        MoveTo(rect.x + 1 + (entered_text.width() as u16), rect.y + 1)
    )
    .ok();

    RenderState::default().show_cursor()
}

pub async fn draw_loading_dialog(
    f: &mut UiFrame<'_>,
    chunk: Rect,
    msg: &str,
    progress: &[Option<&util::Progress>],
) -> RenderState {
    let rect = util::centered_rect_percentage_dir(Direction::Horizontal, 50, chunk);
    let mut height: u16 = 5;
    let mut to_draw: Vec<(f64, String)> = Vec::new();
    for progress in progress {
        if let Some(progress) = progress {
            let msg = progress.get_msg().await;
            let ratio = progress.get().await;

            height += 2;
            if !msg.is_empty() {
                height += 1;
            }
            to_draw.push((ratio, msg));
        };
    }

    let rect = util::centered_rect_dir(Direction::Vertical, height, rect);
    f.render_widget(Clear, rect);
    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain),
        rect,
    );

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            iter::repeat(Constraint::Length(1))
                .take((height - 4) as usize)
                .collect::<Vec<_>>(),
        )
        .margin(2)
        .split(rect);

    f.render_widget(
        Paragraph::new([Text::raw(msg)].iter())
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center),
        layout[0],
    );

    let mut i: usize = 2;

    for (ratio, msg) in to_draw {
        let label = format!("{:.0}%", (ratio * 100.0));
        debug!("Ratio {:?}, msg: {:?}, label: {:?}", ratio, msg, label);
        f.render_widget(
            Gauge::default()
                .style(
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Black)
                        .modifier(Modifier::ITALIC),
                )
                .label(&label)
                .ratio(ratio),
            layout[i],
        );

        if !msg.is_empty() {
            i += 1;
            f.render_widget(
                Paragraph::new([Text::raw(msg)].iter())
                    .style(Style::default().fg(Color::Yellow))
                    .alignment(Alignment::Center),
                layout[i],
            );
        }

        i += 2;
    }

    RenderState::default()
}

pub fn draw_table<D>(
    f: &mut UiFrame<'_>,
    chunk: Rect,
    header: &[&str],
    rows: Vec<Row<D>>,
    widths: &[Constraint],
    title: &str,
    selected: Option<usize>,
) -> RenderState
where
    D: Iterator,
    D::Item: std::fmt::Display,
{
    f.render_widget(Clear, chunk);

    let mut state = TableState::default();
    state.select(selected);

    f.render_stateful_widget(
        Table::new(header.iter(), rows.into_iter())
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Plain),
            )
            .header_style(Style::default().fg(Color::Yellow).modifier(Modifier::BOLD))
            .widths(widths)
            .style(Style::default())
            .highlight_style(Style::default().fg(Color::Blue).modifier(Modifier::BOLD))
            .highlight_symbol(">> ")
            .column_spacing(1)
            .header_gap(0),
        chunk,
        &mut state,
    );
    RenderState::default()
}
