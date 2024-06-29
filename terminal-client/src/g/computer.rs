use std::collections::HashMap;

use super::fs::{Path, Root};
use super::Subprocess;

pub struct Computer<'a> {
	pub cwd: Path,
	pub drive: Option<String>,
	pub root: Root<'a>,
	pub subprocesses: HashMap<String, Subprocess<'a>>,
}