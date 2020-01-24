use super::{ClientState, EcsCompPacket};
use crate::{
    comp, state, sync,
    terrain::{Block, TerrainChunk},
    ChatType,
};
use hashbrown::HashMap;
use vek::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequestStateError {
    Denied,
    Already,
    Impossible,
    WrongMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub description: String,
    pub git_hash: String,
    pub git_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerListUpdate {
    Init(HashMap<u64, String>),
    Add(u64, String),
    Remove(u64),
    Alias(u64, String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMsg {
    InitialSync {
        entity_package: sync::EntityPackage<EcsCompPacket>,
        server_info: ServerInfo,
        time_of_day: state::TimeOfDay,
        world_map: (Vec2<u32>, Vec<u32>),
    },
    PlayerListUpdate(PlayerListUpdate),
    StateAnswer(Result<ClientState, (RequestStateError, ClientState)>),
    /// Trigger cleanup for when the client goes back to the `Registered` state from an ingame
    /// state
    ExitIngameCleanup,
    Ping,
    Pong,
    ChatMsg {
        chat_type: ChatType,
        message: String,
    },
    SetPlayerEntity(u64),
    TimeOfDay(state::TimeOfDay),
    EcsSync(sync::SyncPackage<EcsCompPacket>),
    CreateEntity(sync::EntityPackage<EcsCompPacket>),
    DeleteEntity(u64),
    EntityPos {
        entity: u64,
        pos: comp::Pos,
    },
    EntityVel {
        entity: u64,
        vel: comp::Vel,
    },
    EntityOri {
        entity: u64,
        ori: comp::Ori,
    },
    EntityCharacterState {
        entity: u64,
        character_state: comp::CharacterState,
    },
    InventoryUpdate(comp::Inventory),
    TerrainChunkUpdate {
        key: Vec2<i32>,
        chunk: Result<Box<TerrainChunk>, ()>,
    },
    TerrainBlockUpdates(HashMap<Vec3<i32>, Block>),
    Error(ServerError),
    Disconnect,
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerError {
    TooManyPlayers,
    InvalidAuth,
    //TODO: InvalidAlias,
}

impl ServerMsg {
    pub fn chat(message: String) -> ServerMsg {
        ServerMsg::ChatMsg {
            chat_type: ChatType::Chat,
            message,
        }
    }
    pub fn tell(message: String) -> ServerMsg {
        ServerMsg::ChatMsg {
            chat_type: ChatType::Tell,
            message,
        }
    }
    pub fn game(message: String) -> ServerMsg {
        ServerMsg::ChatMsg {
            chat_type: ChatType::GameUpdate,
            message,
        }
    }
    pub fn broadcast(message: String) -> ServerMsg {
        ServerMsg::ChatMsg {
            chat_type: ChatType::Broadcast,
            message,
        }
    }
    pub fn private(message: String) -> ServerMsg {
        ServerMsg::ChatMsg {
            chat_type: ChatType::Private,
            message,
        }
    }
    pub fn kill(message: String) -> ServerMsg {
        ServerMsg::ChatMsg {
            chat_type: ChatType::Kill,
            message,
        }
    }
}
