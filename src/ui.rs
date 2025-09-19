use ratatui::layout::{Layout, Constraint, Position};
use ratatui::widgets::{
    Block, List, Paragraph, ListItem, HighlightSpacing};
use ratatui::style::{Stylize, Color, Style, Modifier};
use ratatui::text::{Text};
use ratatui::style::palette::tailwind::{SLATE};

use super::App;
use crate::models::*;

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

impl App {
    pub fn render(&mut self, frame: &mut ratatui::Frame) {
        let terminal_width = frame.area().width;

        let main_layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Max(self.calculate_total_display_lines(terminal_width - 2) as u16),
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);

        let [
        top_area, 
        mid_area, 
        input_area, 
        blank_area, 
        bottom_area
    ] = frame.area().layout(&main_layout);

        frame.render_widget(self.title(), top_area);

        let items_cloned = self.todo_list.items.clone();
        // TODO: This should match the prefix length in todo_list (currently "☐ " or "✓ " = 2 chars)
        let list = App::todo_list(items_cloned, terminal_width - 2);
        frame.render_stateful_widget(list, mid_area, &mut self.todo_list.state);

        frame.render_widget(self.input_line(), input_area);

        frame.render_widget(Paragraph::new(String::from("")), blank_area);

        frame.render_widget(self.footer(), bottom_area);

        match self.input_mode {
            InputMode::Normal => {}
            InputMode::Insert => frame.set_cursor_position(Position::new(
                input_area.x + self.character_index as u16 + 1,
                input_area.y + 1,
            )),
        }
    
    }


    pub fn title(&self) -> Paragraph {
        if self.input_mode == InputMode::Insert {
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
                let content = if todo_item.status == Status::Todo {
                    format!("☐ {}", todo_item.todo)
                } else {
                    format!("✓ {}", todo_item.todo)
                };

                let wrapped_lines = wrap_text(&content, width as usize);
                let text = Text::from(wrapped_lines.join("\n"));
                ListItem::new(text)

            })
            .collect();

        return List::new(todo_items)
            .block(Block::new())
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always)
    }

    fn calculate_total_display_lines(&self, width: u16) -> usize {
        self.todo_list.items
            .iter()
            .map(|todo_item| {
                let content = if todo_item.status == Status::Todo {
                    format!("☐ {}", todo_item.todo)
                } else {
                    format!("✓ {}", todo_item.todo)
                };
                wrap_text(&content, width as usize).len()
            })
            .sum()

    }

    fn input_line(&mut self) -> Paragraph {
        Paragraph::new(self.input.clone()) 
            .block(Block::bordered().title_top("New Todo"))
    }

    fn footer(&mut self) -> Paragraph<'static> {
        Paragraph::new("Here's the bottom part")
            .centered()
    }
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String>  {
    textwrap::wrap(text, max_width)
    .into_iter()
    .map(|line| line.to_string())
    .collect()
}
