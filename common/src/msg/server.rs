use super::{ClientState, EcsCompPacket};
use crate::{
    character::CharacterItem,
    comp, state, sync,
    terrain::{Block, TerrainChunk},
    ChatType,
};
use authc::AuthClientError;
use hashbrown::HashMap;
use vek::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub description: String,
    pub git_hash: String,
    pub git_date: String,
    pub auth_provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerListUpdate {
    Init(HashMap<u64, PlayerInfo>),
    Add(u64, PlayerInfo),
    SelectedCharacter(u64, CharacterInfo),
    LevelChange(u64, u32),
    Remove(u64),
    Alias(u64, String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub player_alias: String,
    pub character: Option<CharacterInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterInfo {
    pub name: String,
    pub level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Notification {
    WaypointSaved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMsg {
    InitialSync {
        entity_package: sync::EntityPackage<EcsCompPacket>,
        server_info: ServerInfo,
        time_of_day: state::TimeOfDay,
        world_map: (Vec2<u32>, Vec<u32>),
    },
    /// An error occurred while loading character data
    CharacterDataLoadError(String),
    /// A list of characters belonging to the a authenticated player was sent
    CharacterListUpdate(Vec<CharacterItem>),
    /// An error occured while creating or deleting a character
    CharacterActionError(String),
    PlayerListUpdate(PlayerListUpdate),
    StateAnswer(Result<ClientState, (RequestStateError, ClientState)>),
    /// Trigger cleanup for when the client goes back to the `Registered` state
    /// from an ingame state
    ExitIngameCleanup,
    Ping,
    Pong,
    ChatMsg {
        chat_type: ChatType,
        message: String,
    },
    SetPlayerEntity(u64),
    TimeOfDay(state::TimeOfDay),
    EntitySync(sync::EntitySyncPackage),
    CompSync(sync::CompSyncPackage<EcsCompPacket>),
    CreateEntity(sync::EntityPackage<EcsCompPacket>),
    DeleteEntity(u64),
    InventoryUpdate(comp::Inventory, comp::InventoryUpdateEvent),
    TerrainChunkUpdate {
        key: Vec2<i32>,
        chunk: Result<Box<TerrainChunk>, ()>,
    },
    TerrainBlockUpdates(HashMap<Vec3<i32>, Block>),
    Disconnect,
    Shutdown,
    TooManyPlayers,
    /// Send a popup notification such as "Waypoint Saved"
    Notification(Notification),
    SetViewDistance(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequestStateError {
    RegisterDenied(RegisterError),
    Denied,
    Already,
    Impossible,
    WrongMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RegisterError {
    AlreadyLoggedIn,
    AuthError(String),
    InvalidCharacter,
    NotOnWhitelist,
    //TODO: InvalidAlias,
}

impl From<AuthClientError> for RegisterError {
    fn from(err: AuthClientError) -> Self { Self::AuthError(err.to_string()) }
}

impl ServerMsg {
    // TODO is this needed?
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

    // TODO is this needed?
    pub fn private(message: String) -> ServerMsg {
        ServerMsg::ChatMsg {
            chat_type: ChatType::Private,
            message,
        }
    }

    pub fn group(message: String) -> ServerMsg {
        ServerMsg::ChatMsg {
            chat_type: ChatType::Group,
            message,
        }
    }

    pub fn region(message: String) -> ServerMsg {
        ServerMsg::ChatMsg {
            chat_type: ChatType::Region,
            message,
        }
    }

    pub fn say(message: String) -> ServerMsg {
        ServerMsg::ChatMsg {
            chat_type: ChatType::Say,
            message,
        }
    }

    pub fn faction(message: String) -> ServerMsg {
        ServerMsg::ChatMsg {
            chat_type: ChatType::Faction,
            message,
        }
    }

    pub fn world(message: String) -> ServerMsg {
        ServerMsg::ChatMsg {
            chat_type: ChatType::Chat,
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
