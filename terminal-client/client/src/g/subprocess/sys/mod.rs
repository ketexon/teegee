use super::Subprocess;

mod clear;
mod cmd;
mod help;
mod logout;
mod ssh;
mod which;

pub use clear::*;
pub use cmd::*;
pub use help::*;
pub use logout::*;
pub use ssh::*;
pub use which::*;

pub const DEFAULT: &[(&str, Subprocess)] = &[
    ("cmd", CMD),
    ("logout", LOGOUT),
    ("ssh", SSH),
    ("help", HELP),
    ("which", WHICH),
    ("clear", CLEAR),
];
