pub mod computer;
pub mod fs;
pub mod subprocess;

use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    rc::Rc,
};

pub use computer::Computer;
use computer::{ComputerBuilder, User};
use fs::{File, Node, Path};

use crate::{date, ipc, path};

pub struct Game {
    pub connection: Box<RefCell<dyn ipc::Connection>>,
    pub computers: Vec<Rc<Computer>>,

    current_computer_index: Cell<usize>,
    computer_address_map: HashMap<String, usize>,
    process_queue: RefCell<VecDeque<(String, Vec<String>)>>,
}

impl Game {
    pub fn new(connection: Box<RefCell<dyn ipc::Connection>>) -> Self {
        let default_exes = std::iter::empty()
            .chain(subprocess::sys::DEFAULT)
            .chain(subprocess::fs::DEFAULT)
            .cloned();

        let computers = vec![
            ComputerBuilder::new()
                .name("Plasma_XQ9")
                .users([User::new("root", "123456")])
                .address("64.26.62.54")
                .with_path("bin".to_string())
                .add_dir(&path![], "bin", date!["12 Jan 2024 12:30"])
                .add_file(
                    &path![],
                    "hello1",
                    date!["12 Jan 2024 12:30"],
                    File::new("there"),
                )
                .add_exes_same_date(
                    &path!["bin"],
                    date!["12 Jan 2024 12:30"],
                    default_exes.clone(),
                )
                .build(),
            ComputerBuilder::new()
                .name("Computer1")
                .address("214.7.222.240")
                .with_path("bin".to_string())
                .users([User::new("root", "123456")])
                .add_dir(&path![], "bin", date!["12 Jan 2024 12:30"])
                .add_file(
                    &path![],
                    "hello2",
                    date!["12 Jan 2024 12:30"],
                    File::new("there"),
                )
                .add_exes_same_date(
                    &path!["bin"],
                    date!["12 Jan 2024 12:30"],
                    default_exes.clone(),
                )
                .build(),
        ];
        let computer_address_map: HashMap<String, usize> = computers
            .iter()
            .enumerate()
            .map(|(i, c)| (c.address.clone(), i))
            .collect();

        Self {
            connection,
            computers: computers.into_iter().map(Rc::new).collect(),
            current_computer_index: Cell::new(0),
            computer_address_map,
            process_queue: RefCell::new(Default::default()),
        }
    }

    pub fn current_computer_index(&self) -> usize {
        self.current_computer_index.get()
    }

    pub fn current_computer(&self) -> Rc<Computer> {
        self.computers[self.current_computer_index()].clone()
    }

    pub fn start_exe_from_path<U: Into<Vec<String>>>(
        &self,
        name: &str,
        args: U,
    ) -> Result<std::io::Result<()>, fs::FsError> {
        self.current_computer()
            .which_node(name)
            .ok_or(fs::FsError::NotExecutable)
            .and_then(|node| self.start_exe(node, args))
    }

    pub fn start_exe<U: Into<Vec<String>>>(
        &self,
        node: Node,
        args: U,
    ) -> Result<std::io::Result<()>, fs::FsError> {
        node.as_exe()
            .ok_or(fs::FsError::NotExecutable)
            .map(|node| node.run(self, args.into()))
    }

    pub fn queue_process<U: Into<Vec<String>>>(&self, name: &str, args: U) {
        self.process_queue
            .borrow_mut()
            .push_back((name.to_string(), args.into()));
    }

    pub fn get_queued_process(&self) -> Option<(String, Vec<String>)> {
        self.process_queue.borrow_mut().pop_front()
    }

    #[allow(dead_code)]
    pub fn find_computer_by_address(&self, addr: &String) -> Option<Rc<Computer>> {
        self.computer_address_map
            .get(addr)
            .map(|i| self.computers[*i].clone())
    }

    pub fn change_computers_by_address(&self, addr: &String) -> bool {
        if let Some(index) = self.computer_address_map.get(addr) {
            self.current_computer_index.set(*index);
            return true;
        }
        false
    }
}
