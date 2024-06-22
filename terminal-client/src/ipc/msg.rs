#[derive(Debug, Clone, Copy)]
pub struct MessageHeader {
	pub ty: MessageType,
	pub len: u32,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum MessageType {
	Initialize = 0,
	UnlockDoor = 1,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
	Initialize(InitializeMessage),
	UnlockDoor(UnlockDoorMessage),
}

impl Message {
	pub fn get_type(&self) -> MessageType {
		match self {
			Self::Initialize(_) => MessageType::Initialize,
			Self::UnlockDoor(_) => MessageType::UnlockDoor,
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct InitializeMessage {
	pub index: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct UnlockDoorMessage {
	pub code: [u8; 4],
}

impl From<InitializeMessage> for Message {
	fn from(value: InitializeMessage) -> Self {
		Self::Initialize(value)
	}
}

impl From<UnlockDoorMessage> for Message {
	fn from(value: UnlockDoorMessage) -> Self {
		Self::UnlockDoor(value)
	}
}
