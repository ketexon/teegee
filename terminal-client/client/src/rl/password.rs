use rustyline::{highlight::Highlighter, Completer, Helper, Highlighter, Hinter, Validator};

#[derive(Helper, Validator, Hinter, Completer)]
pub struct PasswordHelper(pub bool);

impl Highlighter for PasswordHelper {
	fn highlight<'l>(&self, line: &'l str, pos: usize) -> std::borrow::Cow<'l, str> {
		if self.0 {
			std::borrow::Cow::Owned(" ".repeat(line.len()))
		}
		else {
			std::borrow::Cow::Borrowed(line)
		}
	}

	fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
		self.0
	}
}