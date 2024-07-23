


use super::Subprocess;

mod cmd;
mod help;
mod logout;
mod which;
mod ssh;

pub use cmd::*;
pub use help::*;
pub use logout::*;
pub use which::*;
pub use ssh::*;

pub const DEFAULT: &[(&str, Subprocess)] = &[
	("cmd", CMD),
	("logout", LOGOUT),
	("ssh", SSH),
	("help", HELP),
	("which", WHICH),
];