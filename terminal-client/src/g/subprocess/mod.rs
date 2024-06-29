use super::Game;

pub type Subprocess<'a> = &'a dyn Fn(&mut Game, Vec<String>) -> std::io::Result<()>;

pub mod fs;
pub mod sys;