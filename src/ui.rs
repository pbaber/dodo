use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::palette::tailwind::SLATE;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, Paragraph};

use crate::models::*;

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

pub fn render_impl(app: &mut crate::app::App, frame: &mut ratatui::Frame) {
    let terminal_width = frame.area().width;

    let main_layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Max(calculate_total_display_lines(app, terminal_width - 2) as u16),
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(1),
    ]);

    let [top_area, mid_area, input_area, blank_area, bottom_area] =
        frame.area().layout(&main_layout);

    frame.render_widget(title(app), top_area);

    let items_cloned = app.todo_list.items.clone();
    // TODO: This should match the prefix length in todo_list (currently "☐ " or "✓ " = 2 chars)
    let list = todo_list(items_cloned, terminal_width - 2);
    frame.render_stateful_widget(list, mid_area, &mut app.todo_list.state);

    frame.render_widget(input_line(app), input_area);

    frame.render_widget(Paragraph::new(String::from("")), blank_area);

    frame.render_widget(footer(), bottom_area);

    match app.input_mode {
        InputMode::Normal => {}
        InputMode::Insert => frame.set_cursor_position(Position::new(
            input_area.x + app.character_index as u16 + 1,
            input_area.y + 1,
        )),
    }
}

pub fn title(app: &crate::app::App) -> Paragraph {
    if app.input_mode == InputMode::Insert {
        Paragraph::new("Insert Mode")
            .bold()
            .style(Style::default().fg(Color::Green))
            .centered()
    } else {
        Paragraph::new("Normal Mode")
            .bold()
            .style(Style::default().fg(Color::Yellow))
            .centered()
    }
}

pub fn todo_list(items: Vec<TodoItem>, width: u16) -> List<'static> {
    let todo_items: Vec<ListItem> = items
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

            let wrapped_lines = wrap_text(&content, width as usize);
            let text = Text::from(wrapped_lines.join("\n"));
            ListItem::new(text)
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

pub fn input_line(app: &crate::app::App) -> Paragraph {
    Paragraph::new(app.input.clone()).block(Block::bordered().title_top("New Todo"))
}

pub fn footer() -> Paragraph<'static> {
    Paragraph::new("j down, k up, c/Enter completed, d delete").centered()
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    textwrap::wrap(text, max_width)
        .into_iter()
        .map(|line| line.to_string())
        .collect()
}
