pub mod msg;

pub use msg::{InitializeMessage, Message, MessageHeader, *};
use msg::{MessageType, UnlockDoorMessage};
use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    time::Duration,
};

const SERVER_TIMEOUT: Duration = Duration::from_secs(3);
const SERVER_ADDRESS: std::net::SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 41987);

#[derive(Debug)]
#[allow(dead_code)]
pub enum ParseError {
    InvalidLength,
    UnknownMessage,
    Io(std::io::Error),
}

trait ReadWrite: Read + Write {}
impl<T: Read + Write> ReadWrite for T {}

struct IoStream;

impl Read for IoStream {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        let mut string = String::new();
        print!("Read {} bytes: ", buf.len());
        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut string)?;
        let bytes = string
            .chars()
            .filter(char::is_ascii_hexdigit)
            .collect::<Vec<char>>()
            .chunks(2)
            .map(|chunk| chunk.iter().collect::<String>())
            .map(|str| u8::from_str_radix(str.as_str(), 16).expect("Invalid hex digit"))
            .take(buf.len())
            .collect::<Vec<u8>>();

        buf.write_all(bytes.as_slice())?;
        Ok(bytes.len())
    }
}

impl Write for IoStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        std::io::stdout().write_all(
            buf.iter()
                .map(|n| format!("{n:x}"))
                .collect::<Vec<String>>()
                .join(" ")
                .as_bytes(),
        )?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stdout().flush()
    }
}

pub trait Connection {
    // fn write<T: Sized>(&mut self, data: &T) -> std::io::Result<()>;
    fn write_message(&mut self, msg: Message) -> std::io::Result<()>;
    fn read_message(&mut self) -> Result<Message, ParseError>;
    fn read_message_expecting(&mut self, _expecting: MessageType) -> Result<Message, ParseError> {
        self.read_message()
    }
}

pub struct StreamConnection {
    stream: Box<dyn ReadWrite>,
    debug: bool,
}

impl Connection for StreamConnection {
    fn write_message(&mut self, msg: Message) -> std::io::Result<()> {
        if self.debug {
            println!("Writing message: {msg:?}");
        }
        match msg {
            Message::Initialize(data) => self.write_message_internal(msg.get_type(), &data),
            Message::UnlockDoor(data) => self.write_message_internal(msg.get_type(), &data),
            Message::SwitchComputer(data) => self.write_message_internal(msg.get_type(), &data),
            Message::PlaySfx(data) => self.write_message_internal(msg.get_type(), &data),
        }
    }

    fn read_message(&mut self) -> Result<Message, ParseError> {
        if self.debug {
            println!("Reading message...");
        }
        self.read_value::<MessageHeader>()
            .map_err(ParseError::Io)
            .and_then(|header| self.parse_message(&header))
    }
}

#[allow(dead_code)]
impl StreamConnection {
    pub fn tcp() -> Option<Self> {
        TcpStream::connect_timeout(&SERVER_ADDRESS, SERVER_TIMEOUT)
            .map(|stream| Self {
                stream: Box::new(stream),
                debug: false,
            })
            .ok()
    }

    pub fn io() -> Self {
        Self {
            stream: Box::new(IoStream),
            debug: true,
        }
    }

    fn write<T: Sized>(&mut self, data: &T) -> std::io::Result<()> {
        self.stream.write(Self::data_as_slice(data)).and(Ok(()))
    }

    fn read_value<T: Sized + Copy>(&mut self) -> std::io::Result<T> {
        self.read_exact(core::mem::size_of::<T>())
            .map(|s| unsafe { (s.as_ptr() as *const T).read() })
    }

    fn read_to_end(&mut self) -> std::io::Result<Vec<u8>> {
        let mut buf: Vec<u8> = Default::default();
        self.stream.read_to_end(&mut buf).and(Ok(buf))
    }

    fn read_exact(&mut self, len: usize) -> std::io::Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
        self.stream.read_exact(&mut buf).and(Ok(buf))
    }

    fn data_as_slice<T: Sized>(data: &T) -> &[u8] {
        let ptr = (data as *const T) as *const u8;
        let size = core::mem::size_of::<T>();
        unsafe { core::slice::from_raw_parts(ptr, size) }
    }

    fn write_message_internal<T: Sized>(
        &mut self,
        ty: MessageType,
        data: &T,
    ) -> std::io::Result<()> {
        self.write(&MessageHeader {
            ty,
            len: core::mem::size_of::<T>() as u32,
        })
        .and_then(|_| self.write(data))
    }

    fn parse_message(&mut self, header: &MessageHeader) -> Result<Message, ParseError> {
        match header.ty {
            MessageType::Initialize => self.parse_message_type::<InitializeMessage>(header),
            MessageType::UnlockDoor => self.parse_message_type::<UnlockDoorMessage>(header),
            #[allow(unreachable_patterns)]
            // the server can send messages that cannot be casted to MessageType
            _ => Err(ParseError::UnknownMessage),
        }
    }

    fn parse_message_type<T: Sized + Copy + Into<Message>>(
        &mut self,
        header: &MessageHeader,
    ) -> Result<Message, ParseError> {
        if header.len as usize != core::mem::size_of::<InitializeMessage>() {
            Err(ParseError::InvalidLength)
        } else {
            self.read_value::<T>()
                .map(|m| m.into())
                .map_err(ParseError::Io)
        }
    }
}

impl Drop for StreamConnection {
    fn drop(&mut self) {
        let _ = self.stream.flush();
    }
}

// pub struct DebugConnection;

// impl Connection for DebugConnection {
// 	fn write_message(&mut self, msg: Message) -> std::io::Result<()> {
// 		println!("write_message {{ {msg:?} }}");
// 		Ok(())
// 	}

// 	fn read_message(&mut self) -> Result<Message, ParseError> {
// 		Self::read_message_impl()
// 	}

// 	fn read_message_expecting(&mut self, expecting: MessageType) -> Result<Message, ParseError> {
// 		println!("read_message_expecting {expecting:?}");
// 		Self::read_message_impl()
// 	}
// }

// #[derive(Debug)]
// pub enum ParseRonMessageError {
// 	Cancelled,
// 	IO(std::io::Error),
// 	Ron(ron::error::SpannedError),
// 	Deserialize(ron::Error),
// 	Reflect,
// }

// impl DebugConnection {
// 	fn read_message_impl() -> Result<Message, ParseError> {
// 		loop {
// 			match Self::parse_message_from_stdio() {
// 				Ok(res) => return Ok(res),
// 				Err(e) => match e {
// 					ParseError::Cancelled => return Err(ParseError::Cancelled),
// 					_ => {
// 						println!("Error parsing: {e:?}. Try again.");
// 					}
// 				}
// 			};
// 		}
// 	}

// 	fn parse_message_from_stdio() -> Result<Message, ParseError> {
// 		let mut bytes = Vec::<u8>::new();
// 		std::io::stdin().read_to_end(&mut bytes).map_err(|e| ParseError::Io(e))?;
// 		if bytes.len() == 0 {
// 			Err(ParseError::Cancelled)
// 		}
// 		else if bytes.starts_with(&[b'0', b'x']) {

// 		}
// 		else {
// 			Self::parse_message_ron(&bytes).map_err(ParseError::Ron)
// 		}
// 	}

// 	fn parse_message_bytes(bytes: &Vec<u8>)

// 	fn parse_message_ron(bytes: &Vec<u8>) -> Result<Message, ParseRonMessageError> {
// 		let mut registry = TypeRegistry::default();
// 		registry.register::<Message>();
// 		let registration = registry.get(TypeId::of::<Message>()).unwrap();

// 		let mut deserializer = ron::Deserializer::from_bytes(&bytes)
// 			.map_err(|e| ParseRonMessageError::Ron(e))?;
// 		let reflect_deserializer = TypedReflectDeserializer::new(registration, &registry);

// 		let output = reflect_deserializer
// 			.deserialize(&mut deserializer)
// 			.map_err(|e| ParseRonMessageError::Deserialize(e))?;

// 		<Message as FromReflect>::from_reflect(&*output)
// 			.ok_or(ParseRonMessageError::Reflect)
// 	}
// }
