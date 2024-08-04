use super::Subprocess;

mod cat;
mod cd;
mod ls;

pub use cat::*;
pub use cd::*;
pub use ls::*;

pub const DEFAULT: &[(&str, Subprocess)] = &[("ls", LS), ("cd", CD), ("cat", CAT)];
