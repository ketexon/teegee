use ratatui::{style::Style, text::{Line, Span, Text}, widgets::Widget};

#[derive(Default, Clone)]
pub struct Markdown<'a> {
    pub text: &'a str,
    pub style: Style,
}

impl<'a> Markdown<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            text,
            style: Style::default(),
        }
    }
}

impl<'a> Into<Text<'a>> for Markdown<'a> {
    fn into(self) -> Text<'a> {
        let mut text = Text::default();
        
        let mut start = 0;
        let mut n_asterisks = 0;
        let mut in_styled = false;
        let mut styled_end: Option<usize> = None;
        let mut last_was_newline = false;
        let mut cur_line = Line::default();
        for (i, ch) in self.text.char_indices() {
            // ignore initial spaces
            if i == start && ch.is_whitespace() {
                start = i;
            }
            // newline on 2 \ns
            if ch == '\n' {
                if last_was_newline {
                    text.push_line(cur_line.clone());
                    cur_line = Line::default();
                    last_was_newline = false;
                }
                else {
                    last_was_newline = true;
                }
            }

            if ch == '*' && n_asterisks == 0 {
                n_asterisks += 1;
                cur_line.push_span(Span::raw(&self.text[start..i]));
            }
            if !in_styled && ch != '*' && n_asterisks > 0 {
                start = i;
                in_styled = true;
            }
            if in_styled && ch == '*' {
                if styled_end.is_none() {
                    styled_end = Some(i); 
                }

                n_asterisks -= 1;
                if n_asterisks == 0 {
                    cur_line.push_span(Span::raw(
                }
            }
        }

        if !cur_line.spans.is_empty() {
            text.push_line(cur_line);
        }

        text
    }
}
