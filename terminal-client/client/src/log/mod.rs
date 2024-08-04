use std::path::PathBuf;
use std::io::Write;

static mut LOG_PATH: Option<PathBuf> = None;

pub fn init(){
	unsafe {
		let log_dir_path = dirs::data_local_dir()
			.map(|dir|
				dir.join("ketexon/termgame/logs")
			);

		LOG_PATH = log_dir_path.clone().map(|dir|
			dir.join(format!("LOG-{}.txt", chrono::Utc::now().to_string().replace(':', "_")))
		);
		if let Some(dir) = log_dir_path {
			let _ = std::fs::create_dir_all(dir);
		}
	}
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::log::log(::std::format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! dbg {
    ($expr:expr) => {
        { if ::std::cfg!(debug_assertions) { $expr } }
    };
}

pub fn log(fmt: std::fmt::Arguments<'_>){
	if let Some(file) = unsafe { LOG_PATH.clone() } {
		let f = std::fs::OpenOptions::new()
			.create(true)	
			.append(true)
			.open(file.clone());

			match f {
				Ok(mut f) => { let _ = writeln!(f, "{}", std::fmt::format(fmt)); }
				Err(e) => { println!("{e:?}"); }
			}
	}
}
