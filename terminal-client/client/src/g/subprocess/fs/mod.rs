
use super::Subprocess;

mod ls;
mod cat;
mod cd;

pub use ls::*;
pub use cat::*;
pub use cd::*;

pub const DEFAULT: &[(&str, Subprocess)] = &[
	("ls", LS),
	("cd", CD),
	("cat", CAT),
];
