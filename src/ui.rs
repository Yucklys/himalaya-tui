use linkify::LinkFinder;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::{
    app::{App, AppState},
    filter::Filter,
    keymap::KeyMode,
};

/// Draw UI
pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Min(18), Constraint::Length(3)].as_ref())
        .split(f.size());
    if app.state.content.0.is_empty() {
        draw_msg_list(f, app, chunks[0]);
    } else {
        draw_content(f, app, chunks[0]);
    }
    draw_commands(f, app, chunks[1]);
}

/// Draw mails list
pub fn draw_msg_list<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let border_style = if app.keymap.mode == KeyMode::Motion {
        Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let selected_style = Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::UNDERLINED)
        .add_modifier(Modifier::BOLD);
    let header_style = Style::default().bg(Color::Blue);

    let header_cells = ["ID", "FLAGS", "SUBJECT", "SENDER", "DATE"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells)
        .style(header_style)
        .height(1)
        .bottom_margin(1);
    let rows: Vec<Vec<String>> = app
        .emails
        .iter()
        .map(|m| {
            let flags = m.flags_string();
            vec![
                m.id.to_string(),
                flags,
                m.subject.clone(),
                m.sender.clone(),
                m.date.clone(),
            ]
        })
        .collect();

    let rows = rows.iter().map(|m| {
        let cells = m.iter().map(|m| m.as_str());
        Row::new(cells).height(1).bottom_margin(1)
    });
    let t = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(selected_style)
        .highlight_symbol(">")
        .column_spacing(2)
        .widths(&[
            Constraint::Length(2),
            Constraint::Length(5),
            Constraint::Percentage(50),
            Constraint::Percentage(20),
            Constraint::Length(10),
        ]);
    f.render_stateful_widget(t, area, &mut app.state.msg_table);
}

/// Draw command line.
pub fn draw_commands<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let style = if app.keymap.mode == KeyMode::Insert {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let block = Block::default().borders(Borders::ALL).border_style(style);
    let mode_style = Style::default()
        .fg(match app.keymap.mode {
            KeyMode::Motion => Color::Blue,
            KeyMode::Insert => Color::Green,
            KeyMode::Review => Color::Yellow,
        })
        .add_modifier(Modifier::BOLD);
    let chunks = Layout::default()
        .constraints([Constraint::Length(12), Constraint::Min(10)].as_ref())
        .direction(Direction::Horizontal)
        .split(area);

    let command = match app.keymap.mode {
        KeyMode::Insert => &app.command_input,
        _ => match app.curr_filter() {
            Some(Filter(filter)) => filter,
            None => &app.command_input,
        },
    };

    let input = Paragraph::new(command.as_str())
        .block(block)
        .style(Style::default().fg(Color::Gray));
    let mode = Paragraph::new(Spans::from(Span::styled(
        app.keymap.mode.to_string(),
        mode_style,
    )))
    .alignment(tui::layout::Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default()),
    );

    f.render_widget(mode, chunks[0]);
    f.render_widget(input, chunks[1]);
}

/// Draw email content
pub fn draw_content<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let block = Block::default().borders(Borders::ALL);
    let mut text = Text::default();
    let finder = LinkFinder::new();
    let AppState {
        content: (content, offset),
        review_flags: flags,
        ..
    } = &mut app.state;

    // clear links stored before
    flags.links.clear();

    for line in content.lines() {
        // buffer string for current line
        let mut line_string = Vec::new();
        // collect all links in the line
        let links: Vec<_> = finder.links(line).collect();

        // label all links iff follow link mode and there is a link in the current line.
        let mut last_link_end = 0;

        for link in links {
            // save link to AppState
            flags.links.push(link.as_str().to_string());

            // split current line into three parts:
            // before the link, the link itself and after the link
            let (_, rest) = line.split_at(last_link_end);
            let (first, _) = rest.split_at(link.start() - last_link_end);

            // add text before link
            line_string.push(Span::from(first));

            // check if links flag is on
            if flags.show_links {
                // add link text with Cyan color
                line_string.push(Span::styled(
                    format!("{} [{}]", link.as_str(), flags.links.len()),
                    Style::default().fg(Color::Cyan),
                ));
            } else {
                line_string.push(Span::raw(link.as_str()));
            }

            // update the index of the end of link
            last_link_end = link.end();
        }

        // add the rest of the line
        let (_, rest) = line.split_at(last_link_end);
        line_string.push(Span::raw(rest));
        text.extend(Text::from(Spans::from(line_string)));
    }

    if flags.show_stats {
        text.extend(Text::raw(format!("Total Links: {}", flags.links.len())));
    }

    let content = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true })
        .scroll((*offset, 0));

    f.render_widget(content, area);
}
