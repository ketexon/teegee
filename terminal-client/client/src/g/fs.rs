use bitmask::bitmask;

use super::subprocess::SubprocessFn;
use std::cell::{Ref, RefCell, RefMut};
use std::fmt::Debug;
use std::rc::{Rc, Weak};

#[macro_export]
macro_rules! date {
    ($date:expr) => {
        ::dateparser::parse($date).unwrap()
    };
}

#[derive(Debug, Clone)]
pub enum FsError {
    AlreadyExists,
    DoesNotExist,
    NotDirectory,
    NotExecutable,
}

pub type FsResult<R = ()> = Result<R, FsError>;

#[derive(Default, Clone, Debug)]
pub struct Path(pub Vec<String>);

#[macro_export]
macro_rules! path {
    () => { Path::new(vec![]) };
    ($elem:expr; $n:expr) => { Path::new(vec![$elem; $n]) };
    ($($x:expr),+ $(,)?) => { Path::new(vec![$($x.to_string()),+]) };
}

impl Path {
    pub fn new<T: Into<Vec<String>>>(path: T) -> Self {
        Self(path.into())
    }

    pub fn parse(cwd: &Path, relative: &str) -> Self {
        let mut new_path = if relative.starts_with('/') {
            Vec::new()
        } else {
            cwd.0.clone()
        };

        new_path.extend(relative.split('/').map(|s| s.to_string()));

        Self(new_path).normalized()
    }

    pub fn join(mut self, other: &Path) -> Self {
        self.0.extend(other.0.clone());
        Path(self.0)
    }

    pub fn normalized(&self) -> Self {
        let mut new_path: Vec<String> = Default::default();
        for sub in &self.0 {
            if sub.is_empty() || sub == "." {
                continue;
            } else if sub == ".." {
                new_path.pop();
            } else {
                new_path.push(sub.to_string());
            }
        }
        Self(new_path)
    }

    pub fn parent(&self) -> Self {
        if self.0.is_empty() {
            Default::default()
        } else {
            Self(self.0[0..self.0.len() - 1].to_vec())
        }
    }

    pub fn basename(&self) -> Option<String> {
        self.0.last().cloned()
    }
}

impl core::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(["/".to_string(), self.0.join("/")].join("").as_str())
    }
}

#[derive(Debug, Clone)]
pub struct File {
    pub content: String,
}

impl File {
    pub fn new<U: ToString>(content: U) -> Self {
        Self {
            content: content.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dir {
    pub children: Vec<Node>,
}

#[allow(dead_code)]
impl Dir {
    pub fn empty() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn new<U: Into<Vec<Node>>>(children: U) -> Self {
        let mut out = Self {
            children: children.into(),
        };
        out.sort();
        out
    }

    pub fn binary_search<T: ToString>(&self, name: T) -> Result<usize, usize> {
        self.children
            .binary_search_by(|entry| entry.borrow().name.cmp(&name.to_string()))
    }

    pub fn get_child_index<T: ToString>(&self, name: T) -> Option<usize> {
        self.binary_search(name).ok()
    }

    pub fn get_child<T: ToString>(&self, name: T) -> Option<Node> {
        self.get_child_index(name)
            .map(|index| self.children[index].clone())
    }

    pub fn add_child(&mut self, node: Node) -> FsResult {
        let name: String = node.borrow().name.clone();
        match self.binary_search(name) {
            Ok(_) => Err(FsError::AlreadyExists),
            Err(index) => {
                self.children.insert(index, node);
                Ok(())
            }
        }
    }

    fn sort(&mut self) {
        self.children
            .sort_by(|a, b| (*a.0).borrow().name.cmp(&(*b.0).borrow().name));
    }
}

#[derive(Clone)]
pub struct Root {
    pub node: Node,
}

impl Root {
    pub fn new<U>(children: U) -> Self
    where
        U: Into<Vec<Node>>,
    {
        Self {
            node: Node::dir(
                "",
                dateparser::parse("15 Jan 2023 00:00").expect("Could not parse date"),
                Dir::new(children.into()),
            ),
        }
    }

    pub fn get_node(&self, path: &Path) -> Option<Node> {
        self.node.get_node(path)
    }

    pub fn get_dir(&self, path: &Path) -> Option<Dir> {
        self.get_node(path).and_then(|node| node.as_dir())
    }
}

pub type Executable = &'static dyn SubprocessFn;

#[derive(Debug, Clone)]
pub enum NodeContent {
    File(File),
    Dir(Dir),
    Executable(Executable),
}

bitmask! {
    pub mask Security: u8 where flags SecurityMode {
        Read 	= 1,
        Write 	= 2,
        Execute = 4,
        RW 		= 1 | 2,
        All 	= 7,
    }
}

impl std::fmt::Display for Security {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            if self.contains(SecurityMode::Read) {
                'r'
            } else {
                '-'
            },
            if self.contains(SecurityMode::Write) {
                'w'
            } else {
                '-'
            },
            if self.contains(SecurityMode::Execute) {
                'x'
            } else {
                '-'
            }
        )
    }
}

impl Debug for Security {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Security {{ {self} }}")
    }
}

pub type NodeDateTime = chrono::DateTime<chrono::Utc>;

#[derive(Debug, Clone)]
pub struct Node(pub Rc<RefCell<NodeData>>);

#[derive(Debug, Clone, Default)]
pub struct WeakNode(pub Weak<RefCell<NodeData>>);

#[derive(Debug, Clone)]
pub struct NodeData {
    pub name: String,
    pub date: NodeDateTime,
    pub content: NodeContent,

    pub owner_security: Security,
    pub other_security: Security,
}

#[allow(dead_code)]
impl Node {
    pub fn data(&self) -> Ref<'_, NodeData> {
        (*self.0).borrow()
    }

    pub fn data_mut(&self) -> RefMut<'_, NodeData> {
        (*self.0).borrow_mut()
    }

    pub fn borrow(&self) -> Ref<'_, NodeData> {
        self.data()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, NodeData> {
        self.data_mut()
    }

    pub fn downgrade(&self) -> WeakNode {
        WeakNode(Rc::downgrade(&self.0))
    }

    pub fn new(data: NodeData) -> Self {
        Self(Rc::new(RefCell::new(data)))
    }

    pub fn dir<T: ToString>(name: T, date: NodeDateTime, dir: Dir) -> Self {
        Self::new(NodeData {
            name: name.to_string(),
            date,
            content: NodeContent::Dir(dir),

            owner_security: SecurityMode::RW.into(),
            other_security: SecurityMode::Read.into(),
        })
    }

    pub fn file<T: ToString>(name: T, date: NodeDateTime, file: File) -> Self {
        Self::new(NodeData {
            name: name.to_string(),
            date,
            content: NodeContent::File(file),

            owner_security: SecurityMode::RW.into(),
            other_security: SecurityMode::Read.into(),
        })
    }

    pub fn exe<T: ToString>(
        name: T,
        date: NodeDateTime,
        subprocess: &'static dyn SubprocessFn,
    ) -> Self {
        Self::new(NodeData {
            name: name.to_string(),
            date,
            content: NodeContent::Executable(subprocess),

            owner_security: SecurityMode::Execute.into(),
            other_security: SecurityMode::Execute.into(),
        })
    }

    pub fn is_dir(&self) -> bool {
        matches!(&self.borrow().content, NodeContent::Dir(_))
    }

    pub fn is_file(&self) -> bool {
        matches!(&self.borrow().content, NodeContent::File(_))
    }

    pub fn is_exe(&self) -> bool {
        matches!(&self.borrow().content, NodeContent::Executable(_))
    }

    pub fn as_dir(&self) -> Option<Dir> {
        match &self.borrow().content {
            NodeContent::Dir(d) => Some(d.clone()),
            _ => None,
        }
    }

    pub fn as_file(&self) -> Option<File> {
        match &self.borrow().content {
            NodeContent::File(f) => Some(f.clone()),
            _ => None,
        }
    }

    pub fn as_exe(&self) -> Option<Executable> {
        match &self.borrow().content {
            NodeContent::Executable(e) => Some(*e),
            _ => None,
        }
    }

    pub fn get_node(&self, path: &Path) -> Option<Node> {
        match &path.0[..] {
            [a, b @ ..] => match &self.borrow().content {
                NodeContent::Dir(dir) => dir
                    .get_child(a)
                    .and_then(|child| child.get_node(&Path::new(b))),
                _ => None,
            },
            _ => Some(self.clone()),
        }
    }

    pub fn add_child(&self, node: Node) -> FsResult {
        match &mut self.borrow_mut().content {
            NodeContent::Dir(dir) => dir.add_child(node),
            _ => Err(FsError::NotDirectory),
        }
    }

    pub fn add_node(&self, dir: &Path, node: Node) -> FsResult<()> {
        match self.get_node(dir) {
            Some(dir) => dir.add_child(node).and(Ok(())),
            None => Err(FsError::DoesNotExist),
        }
    }
}

#[allow(dead_code)]
impl WeakNode {
    pub fn upgrade(&self) -> Option<Node> {
        Weak::upgrade(&self.0).map(Node)
    }
}
