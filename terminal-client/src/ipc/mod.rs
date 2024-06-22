pub mod msg;

use std::{io::{Read, Write}, net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream}, time::Duration};
use msg::{MessageType, UnlockDoorMessage};
pub use msg::{InitializeMessage, Message, MessageHeader};


const SERVER_TIMEOUT: Duration = Duration::from_secs(3);
const SERVER_ADDRESS: std::net::SocketAddr = SocketAddr::new(
	IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
	41987
);

pub struct Connection {
	stream: std::net::TcpStream,
}


impl Connection {
	pub fn new() -> Option<Self> {
		TcpStream::connect_timeout(&SERVER_ADDRESS, SERVER_TIMEOUT)
			.map(|stream| Self { stream })
			.ok()
	}

	fn data_as_slice<T: Sized>(data: &T) -> &[u8] {
		let ptr = (data as *const T) as *const u8;
		let size = core::mem::size_of::<T>();
		unsafe {
			core::slice::from_raw_parts(ptr, size)
		}
	}

	pub fn write<T: Sized>(&mut self, data: &T) -> std::io::Result<()>  {
		self.stream.write(Self::data_as_slice(data)).and(Ok(()))
	}

	pub fn write_message(&mut self, msg: Message) -> std::io::Result<()> {
		match msg {
			Message::Initialize(data) => self.write_message_internal(msg.get_type(), &data),
			Message::UnlockDoor(data) => self.write_message_internal(msg.get_type(), &data),
		}
	}

	fn write_message_internal<T: Sized>(&mut self, ty: MessageType, data: &T) -> std::io::Result<()> {
		self.write(&MessageHeader {
			ty,
			len: core::mem::size_of::<T>() as u32,
		}).and_then(|_| self.write(data))

	}

	pub fn read_to_end(&mut self) -> std::io::Result<Vec<u8>> {
		let mut buf: Vec<u8> = Default::default();
		self.stream.read_to_end(&mut buf).and(Ok(buf))
	}

	pub fn read_exact(&mut self, len: usize) -> std::io::Result<Vec<u8>> {
		let mut buf = vec![0u8; len];
		self.stream.read_exact(&mut buf).and(Ok(buf))
	}

	pub fn read_value<T: Sized + Copy>(&mut self) -> std::io::Result<T> {
		self.read_exact(core::mem::size_of::<T>())
			.map(|s| unsafe { (s.as_ptr() as *const T).read() })
	}

	pub fn read_message(&mut self) -> Result<Message, ParseError> {
		self.read_value::<MessageHeader>()
			.or_else(|e| Err(ParseError::IoError(e)))
			.and_then(|header| self.parse_message(&header))
	}

	fn parse_message(&mut self, header: &MessageHeader) -> Result<Message, ParseError> {
		match header.ty {
			MessageType::Initialize => self.parse_message_type::<InitializeMessage>(&header),
			MessageType::UnlockDoor => self.parse_message_type::<UnlockDoorMessage>(&header),
			#[allow(unreachable_patterns)] // the server can send messages that cannot be casted to MessageType
			_ => Err(ParseError::UnknownMessage)
		}
	}

	fn parse_message_type<T: Sized + Copy + Into<Message>>(&mut self, header: &MessageHeader) -> Result<Message, ParseError> {
		if header.len as usize != core::mem::size_of::<InitializeMessage>() {
			Err(ParseError::InvalidLength)
		}
		else {
			self.read_value::<T>()
				.map(|m| m.into())
				.or_else(|e| Err(ParseError::IoError(e)))
		}
	}
}

impl Drop for Connection {
	fn drop(&mut self) {
		let _ = self.stream.flush();
	}
}

#[derive(Debug)]
pub enum ParseError {
	InvalidFormat,
	InvalidLength,
	UnknownMessage,
	IoError(std::io::Error),
}