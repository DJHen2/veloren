use crate::{
    client::{Client, RegionSubscription},
    Server,
};
use common::{
    assets, comp,
    msg::ServerMsg,
    sync::{Uid, WorldSyncExt},
};
use log::error;
use specs::{world::WorldExt, Entity as EcsEntity};

pub fn handle_mount(server: &mut Server, mounter: EcsEntity, mountee: EcsEntity) {
    let state = server.state_mut();

    if state
        .ecs()
        .read_storage::<comp::Mounting>()
        .get(mounter)
        .is_none()
    {
        let not_mounting_yet = if let Some(comp::MountState::Unmounted) = state
            .ecs()
            .read_storage::<comp::MountState>()
            .get(mountee)
            .cloned()
        {
            true
        } else {
            false
        };

        if not_mounting_yet {
            if let (Some(mounter_uid), Some(mountee_uid)) = (
                state.ecs().uid_from_entity(mounter),
                state.ecs().uid_from_entity(mountee),
            ) {
                state.write_component(mountee, comp::MountState::MountedBy(mounter_uid.into()));
                state.write_component(mounter, comp::Mounting(mountee_uid.into()));
            }
        }
    }
}

pub fn handle_unmount(server: &mut Server, mounter: EcsEntity) {
    let state = server.state_mut();
    let mountee_entity = state
        .ecs()
        .write_storage::<comp::Mounting>()
        .get(mounter)
        .and_then(|mountee| state.ecs().entity_from_uid(mountee.0.into()));
    if let Some(mountee_entity) = mountee_entity {
        state
            .ecs()
            .write_storage::<comp::MountState>()
            .get_mut(mountee_entity)
            .map(|ms| *ms = comp::MountState::Unmounted);
    }
    state.delete_component::<comp::Mounting>(mounter);
}

pub fn handle_possess(server: &Server, possessor_uid: Uid, possesse_uid: Uid) {
    let state = &server.state;
    let ecs = state.ecs();
    if let (Some(possessor), Some(possesse)) = (
        ecs.entity_from_uid(possessor_uid.into()),
        ecs.entity_from_uid(possesse_uid.into()),
    ) {
        // You can't possess other players
        let mut clients = ecs.write_storage::<Client>();
        if clients.get_mut(possesse).is_none() {
            if let Some(mut client) = clients.remove(possessor) {
                client.notify(ServerMsg::SetPlayerEntity(possesse_uid.into()));
                clients.insert(possesse, client).err().map(|e| {
                    error!(
                        "Error inserting client component during possession: {:?}",
                        e
                    )
                });
                // Put possess item into loadout
                let mut loadouts = ecs.write_storage::<comp::Loadout>();
                let loadout = loadouts
                    .entry(possesse)
                    .expect("Could not read loadouts component while possessing")
                    .or_insert(comp::Loadout::default());

                let item = assets::load_expect_cloned::<comp::Item>("common.items.debug.possess");
                if let comp::ItemKind::Tool(tool) = item.kind {
                    loadout.active_item = Some(comp::ItemConfig {
                        item,
                        primary_ability: tool.get_abilities().get(0).cloned(),
                        secondary_ability: None,
                        block_ability: None,
                        dodge_ability: None,
                    });
                }

                // Move player component
                {
                    let mut players = ecs.write_storage::<comp::Player>();
                    if let Some(player) = players.remove(possessor) {
                        players.insert(possesse, player).err().map(|e| {
                            error!(
                                "Error inserting player component during possession: {:?}",
                                e
                            )
                        });
                    }
                }
                // Transfer region subscription
                {
                    let mut subscriptions = ecs.write_storage::<RegionSubscription>();
                    if let Some(s) = subscriptions.remove(possessor) {
                        subscriptions.insert(possesse, s).err().map(|e| {
                            error!(
                                "Error inserting subscription component during possession: {:?}",
                                e
                            )
                        });
                    }
                }
                // Remove will of the entity
                ecs.write_storage::<comp::Agent>().remove(possesse);
                // Reset controller of former shell
                ecs.write_storage::<comp::Controller>()
                    .get_mut(possessor)
                    .map(|c| c.reset());
                // Transfer admin powers
                {
                    let mut admins = ecs.write_storage::<comp::Admin>();
                    if let Some(admin) = admins.remove(possessor) {
                        admins.insert(possesse, admin).err().map(|e| {
                            error!("Error inserting admin component during possession: {:?}", e)
                        });
                    }
                }
                // Transfer waypoint
                {
                    let mut waypoints = ecs.write_storage::<comp::Waypoint>();
                    if let Some(waypoint) = waypoints.remove(possessor) {
                        waypoints.insert(possesse, waypoint).err().map(|e| {
                            error!(
                                "Error inserting waypoint component during possession {:?}",
                                e
                            )
                        });
                    }
                }
            }
        }
    }
}
