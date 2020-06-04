use crate::{comp, terrain::block::Block};
use vek::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMsg {
    Register {
        view_distance: Option<u32>,
        token_or_username: String,
    },
    RequestCharacterList,
    CreateCharacter {
        alias: String,
        tool: Option<String>,
        body: comp::Body,
    },
    DeleteCharacter(i32),
    Character {
        character_id: i32,
        body: comp::Body,
    },
    /// Request `ClientState::Registered` from an ingame state
    ExitIngame,
    /// Request `ClientState::Spectator` from a registered or ingame state
    Spectate,
    ControllerInputs(comp::ControllerInputs),
    ControlEvent(comp::ControlEvent),
    ControlAction(comp::ControlAction),
    SetViewDistance(u32),
    BreakBlock(Vec3<i32>),
    PlaceBlock(Vec3<i32>, Block),
    Ping,
    Pong,
    ChatMsg {
        message: String,
    },
    PlayerPhysics {
        pos: comp::Pos,
        vel: comp::Vel,
        ori: comp::Ori,
    },
    TerrainChunkRequest {
        key: Vec2<i32>,
    },
    Disconnect,
    Terminate,
}
