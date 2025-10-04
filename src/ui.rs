use crate::models::*;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::palette::tailwind::SLATE;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, Paragraph};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

pub fn render_impl(app: &mut crate::app::App, frame: &mut ratatui::Frame) {
    let terminal_width = frame.area().width;

    let main_layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Max(calculate_total_display_lines(app, terminal_width - 2) as u16),
        Constraint::Fill(1),
        Constraint::Length(1),
    ]);

    let areas = main_layout.split(frame.area());
    let [top_area, mid_area, blank_area, bottom_area] = [areas[0], areas[1], areas[2], areas[3]];

    frame.render_widget(title(app), top_area);

    let list = todo_list(app, terminal_width - 2);

    frame.render_stateful_widget(list, mid_area, &mut app.todo_list.state);

    frame.render_widget(Paragraph::new(String::from("")), blank_area);

    frame.render_widget(footer(), bottom_area);

    let area = frame.area();
    let popup_width = 60;
    let popup_height = 10;
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_rect = ratatui::layout::Rect::new(x, y, popup_width, popup_height);

    let test_block = Block::new()
        .borders(ratatui::widgets::Borders::ALL)
        .title("Test Popup");

    frame.render_widget(ratatui::widgets::Clear, popup_rect);
    frame.render_widget(&app.textarea, popup_rect);
}

pub fn title(app: &crate::app::App) -> Paragraph {
    if app.input_mode == InputMode::Insert {
        if app.editing_index.is_some() {
            Paragraph::new("Editing")
                .bold()
                .style(Style::default().fg(Color::Cyan))
                .centered()
        } else {
            Paragraph::new("Insert Mode")
                .bold()
                .style(Style::default().fg(Color::Green))
                .centered()
        }
    } else {
        Paragraph::new("Normal Mode")
            .bold()
            .style(Style::default().fg(Color::Yellow))
            .centered()
    }
}

fn indent_span(todo_item: &TodoItem) -> Span<'static> {
    if todo_item.parent_id.is_some() {
        Span::raw("  ")
    } else {
        Span::raw("")
    }
}

fn checkbox_span(todo_item: &TodoItem) -> Span<'static> {
    if todo_item.completed_at.is_none() {
        Span::raw("☐ ")
    } else {
        Span::raw("✓ ")
    }
}

pub fn todo_list(app: &crate::app::App, width: u16) -> List<'static> {
    let todo_items: Vec<ListItem> = app
        .todo_list
        .items
        .iter()
        .map(|todo_item| {
            let indent = indent_span(todo_item);
            let checkbox = checkbox_span(todo_item);
            let prefix_width = indent.width() + checkbox.width();

            let text_width = (width as usize).saturating_sub(prefix_width);
            // get the text content for wrapping
            let text_content = todo_item.todo.clone();

            let wrapped = wrap_text(&text_content, text_width);

            // Create Lines
            let lines: Vec<Line> = wrapped
                .iter()
                .enumerate()
                .map(|(i, line)| {
                    if i == 0 {
                        Line::from(vec![
                            indent.clone(),
                            checkbox.clone(),
                            Span::raw(line.to_string()),
                        ])
                    } else {
                        Line::from(vec![
                            indent.clone(),
                            Span::raw("  ".to_string()),
                            Span::raw(line.to_string()),
                        ])
                    }
                })
                .collect();

            ListItem::new(lines)
        })
        .collect();

    List::new(todo_items)
        .block(Block::new())
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol(">")
        .highlight_spacing(HighlightSpacing::Always)
}

pub fn calculate_total_display_lines(app: &crate::app::App, width: u16) -> usize {
    app.todo_list
        .items
        .iter()
        .map(|todo_item| {
            let indent = if todo_item.parent_id.is_some() {
                "  "
            } else {
                ""
            };
            let content = if todo_item.completed_at.is_none() {
                format!("{}☐ {}", indent, todo_item.todo)
            } else {
                format!("{}✓ {}", indent, todo_item.todo)
            };
            wrap_text(&content, width as usize).len()
        })
        .sum()
}

pub fn footer() -> Paragraph<'static> {
    Paragraph::new("j down, k up, e edit, c/Enter completed, d delete").centered()
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    textwrap::wrap(text, max_width)
        .into_iter()
        .map(|line| line.to_string())
        .collect()
}
