use crate::Server;
use common::event::{EventBus, ServerEvent};
use entity_creation::{
    handle_create_character, handle_create_npc, handle_create_waypoint, handle_shoot,
};
use entity_manipulation::{
    handle_damage, handle_destroy, handle_explosion, handle_land_on_ground, handle_respawn,
};
use interaction::{handle_lantern, handle_mount, handle_possess, handle_unmount};
use inventory_manip::handle_inventory;
use player::{handle_client_disconnect, handle_exit_ingame};
use specs::{Entity as EcsEntity, WorldExt};

mod entity_creation;
mod entity_manipulation;
mod interaction;
mod inventory_manip;
mod player;

pub enum Event {
    ClientConnected {
        entity: EcsEntity,
    },
    ClientDisconnected {
        entity: EcsEntity,
    },
    Chat {
        entity: Option<EcsEntity>,
        msg: String,
    },
}

impl Server {
    pub fn handle_events(&mut self) -> Vec<Event> {
        let mut frontend_events = Vec::new();

        let mut requested_chunks = Vec::new();
        let mut chat_commands = Vec::new();

        let events = self
            .state
            .ecs()
            .read_resource::<EventBus<ServerEvent>>()
            .recv_all();

        for event in events {
            match event {
                ServerEvent::Explosion { pos, power, owner } => {
                    handle_explosion(&self, pos, power, owner)
                },
                ServerEvent::Shoot {
                    entity,
                    dir,
                    body,
                    light,
                    projectile,
                    gravity,
                } => handle_shoot(self, entity, dir, body, light, projectile, gravity),
                ServerEvent::Damage { uid, change } => handle_damage(&self, uid, change),
                ServerEvent::Destroy { entity, cause } => handle_destroy(self, entity, cause),
                ServerEvent::InventoryManip(entity, manip) => handle_inventory(self, entity, manip),
                ServerEvent::Respawn(entity) => handle_respawn(&self, entity),
                ServerEvent::LandOnGround { entity, vel } => {
                    handle_land_on_ground(&self, entity, vel)
                },
                ServerEvent::ToggleLantern(entity) => handle_lantern(self, entity),
                ServerEvent::Mount(mounter, mountee) => handle_mount(self, mounter, mountee),
                ServerEvent::Unmount(mounter) => handle_unmount(self, mounter),
                ServerEvent::Possess(possessor_uid, possesse_uid) => {
                    handle_possess(&self, possessor_uid, possesse_uid)
                },
                ServerEvent::CreateCharacter {
                    entity,
                    name,
                    body,
                    main,
                } => handle_create_character(self, entity, name, body, main),
                ServerEvent::ExitIngame { entity } => handle_exit_ingame(self, entity),
                ServerEvent::CreateNpc {
                    pos,
                    stats,
                    loadout,
                    body,
                    agent,
                    alignment,
                    scale,
                } => handle_create_npc(self, pos, stats, loadout, body, agent, alignment, scale),
                ServerEvent::CreateWaypoint(pos) => handle_create_waypoint(self, pos),
                ServerEvent::ClientDisconnect(entity) => {
                    frontend_events.push(handle_client_disconnect(self, entity))
                },

                ServerEvent::ChunkRequest(entity, key) => {
                    requested_chunks.push((entity, key));
                },
                ServerEvent::ChatCmd(entity, cmd) => {
                    chat_commands.push((entity, cmd));
                },
            }
        }

        // Generate requested chunks.
        for (entity, key) in requested_chunks {
            self.generate_chunk(entity, key);
        }

        for (entity, cmd) in chat_commands {
            self.process_chat_cmd(entity, cmd);
        }

        frontend_events
    }
}
