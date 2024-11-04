use crate::g::computer::ComputerId;
use bevy_reflect::Reflect;

#[derive(Debug, Clone, Copy)]
pub struct MessageHeader {
    pub ty: MessageType,
    pub len: u32,
}

#[repr(u32)]
#[derive(ToPrimitive, FromPrimitive, Debug, Clone, Copy)]
pub enum MessageType {
    Initialize = 0,
    InitializeOS = 4,
    UnlockDoor = 1,
    SwitchComputer = 2,
    PlaySfx = 3,
}

#[derive(Reflect, Debug, Clone, Copy)]
#[type_path = "c"]
pub enum Message {
    Initialize(InitializeMessage),
    InitializeOS(InitializeOSMessage),
    UnlockDoor(UnlockDoorMessage),
    SwitchComputer(SwitchComputerMessage),
    PlaySfx(PlaySfxMessage),
}

impl Message {
    pub fn get_type(&self) -> MessageType {
        match self {
            Self::Initialize(_) => MessageType::Initialize,
            Self::InitializeOS(_) => MessageType::InitializeOS,
            Self::UnlockDoor(_) => MessageType::UnlockDoor,
            Self::SwitchComputer(_) => MessageType::SwitchComputer,
            Self::PlaySfx(_) => MessageType::PlaySfx,
        }
    }
}

#[derive(Reflect, ToPrimitive, FromPrimitive, Debug, Clone, Copy)]
#[repr(u32)]
#[type_path = "c"]
pub enum TerminalType {
    OS = 0,
    Pinpad = 1,
}

#[derive(Reflect, Debug, Clone, Copy)]
#[type_path = "c"]
pub struct InitializeMessage {
    pub terminal_type: TerminalType,
}

#[derive(Reflect, Debug, Clone, Copy)]
#[type_path = "c"]
pub struct InitializeOSMessage {
    pub computer_id: ComputerId,
}

#[derive(Reflect, Debug, Clone, Copy)]
pub struct UnlockDoorMessage {
    pub code: [u8; 4],
}

#[derive(Reflect, Debug, Clone, Copy)]
pub struct SwitchComputerMessage {
    pub new_id: ComputerId,
}

#[derive(Reflect, Debug, Clone, Copy)]
pub struct PlaySfxMessage {
    pub id: u32,
}

impl From<InitializeMessage> for Message {
    fn from(value: InitializeMessage) -> Self {
        Self::Initialize(value)
    }
}

impl From<InitializeOSMessage> for Message {
    fn from(value: InitializeOSMessage) -> Self {
        Self::InitializeOS(value)
    }
}

impl From<UnlockDoorMessage> for Message {
    fn from(value: UnlockDoorMessage) -> Self {
        Self::UnlockDoor(value)
    }
}

impl From<SwitchComputerMessage> for Message {
    fn from(value: SwitchComputerMessage) -> Self {
        Self::SwitchComputer(value)
    }
}

impl From<PlaySfxMessage> for Message {
    fn from(value: PlaySfxMessage) -> Self {
        Self::PlaySfx(value)
    }
}
