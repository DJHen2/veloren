use crate::{comp, state};
use serde_derive::{Deserialize, Serialize};
use std::marker::PhantomData;

// Automatically derive From<T> for EcsResPacket
// for each variant EcsResPacket::T(T).
sphynx::sum_type! {
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum EcsResPacket {
        Time(state::Time),
        TimeOfDay(state::TimeOfDay),
    }
}
impl sphynx::ResPacket for EcsResPacket {}
// Automatically derive From<T> for EcsCompPacket
// for each variant EcsCompPacket::T(T.)
sphynx::sum_type! {
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum EcsCompPacket {
        Pos(comp::Pos),
        Vel(comp::Vel),
        Ori(comp::Ori),
        Body(comp::Body),
        Player(comp::Player),
        Stats(comp::Stats),
    }
}
// Automatically derive From<T> for EcsCompPhantom
// for each variant EcsCompPhantom::T(PhantomData<T>).
sphynx::sum_type! {
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum EcsCompPhantom {
        Pos(PhantomData<comp::Pos>),
        Vel(PhantomData<comp::Vel>),
        Ori(PhantomData<comp::Ori>),
        Body(PhantomData<comp::Body>),
        Player(PhantomData<comp::Player>),
        Stats(PhantomData<comp::Stats>),
    }
}
impl sphynx::CompPacket for EcsCompPacket {
    type Phantom = EcsCompPhantom;
}
