use crate::{client::Client, Server};
use common::{
    comp::{
        self,
        group::{self, GroupManager},
        ChatType, GroupManip,
    },
    msg::ServerMsg,
    sync,
    sync::WorldSyncExt,
};
use specs::world::WorldExt;
use tracing::warn;

// TODO: turn chat messages into enums
pub fn handle_group(server: &mut Server, entity: specs::Entity, manip: GroupManip) {
    let state = server.state_mut();

    match manip {
        GroupManip::Invite(uid) => {
            let mut clients = state.ecs().write_storage::<Client>();
            let invitee = match state.ecs().entity_from_uid(uid.into()) {
                Some(t) => t,
                None => {
                    // Inform of failure
                    if let Some(client) = clients.get_mut(entity) {
                        client.notify(
                            ChatType::Meta
                                .server_msg("Invite failed, target does not exist.".to_owned()),
                        );
                    }
                    return;
                },
            };

            let uids = state.ecs().read_storage::<sync::Uid>();

            // Check if entity is trying to invite themselves to a group
            if uids
                .get(entity)
                .map_or(false, |inviter_uid| *inviter_uid == uid)
            {
                warn!("Entity tried to invite themselves into a group");
                return;
            }

            let alignments = state.ecs().read_storage::<comp::Alignment>();
            let agents = state.ecs().read_storage::<comp::Agent>();
            let mut already_has_invite = false;
            let mut add_to_group = false;
            // If client comp
            if let (Some(client), Some(inviter_uid)) = (clients.get_mut(invitee), uids.get(entity))
            {
                if client.invited_to_group.is_some() {
                    already_has_invite = true;
                } else {
                    client.notify(ServerMsg::GroupInvite(*inviter_uid));
                    client.invited_to_group = Some(entity);
                }
            // Would be cool to do this in agent system (e.g. add an invited
            // component to replace the field on Client)
            // TODO: move invites to component and make them time out
            } else if matches!(
                (alignments.get(invitee), agents.get(invitee)),
                (Some(comp::Alignment::Npc), Some(_))
            ) {
                add_to_group = true;
                // Wipe agent state
                drop(agents);
                let _ = state
                    .ecs()
                    .write_storage()
                    .insert(invitee, comp::Agent::default());
            } else if let Some(client) = clients.get_mut(entity) {
                client.notify(ChatType::Meta.server_msg("Invite rejected.".to_owned()));
            }

            if already_has_invite {
                // Inform inviter that there is already an invite
                if let Some(client) = clients.get_mut(entity) {
                    client.notify(
                        ChatType::Meta
                            .server_msg("This player already has a pending invite.".to_owned()),
                    );
                }
            }

            if add_to_group {
                let mut group_manager = state.ecs().write_resource::<GroupManager>();
                group_manager.add_group_member(
                    entity,
                    invitee,
                    &state.ecs().entities(),
                    &mut state.ecs().write_storage(),
                    &alignments,
                    &uids,
                    |entity, group_change| {
                        clients
                            .get_mut(entity)
                            .and_then(|c| {
                                group_change
                                    .try_map(|e| uids.get(e).copied())
                                    .map(|g| (g, c))
                            })
                            .map(|(g, c)| c.notify(ServerMsg::GroupUpdate(g)));
                    },
                );
            }
        },
        GroupManip::Accept => {
            let mut clients = state.ecs().write_storage::<Client>();
            let uids = state.ecs().read_storage::<sync::Uid>();
            if let Some(inviter) = clients
                .get_mut(entity)
                .and_then(|c| c.invited_to_group.take())
            {
                let mut group_manager = state.ecs().write_resource::<GroupManager>();
                group_manager.add_group_member(
                    inviter,
                    entity,
                    &state.ecs().entities(),
                    &mut state.ecs().write_storage(),
                    &state.ecs().read_storage(),
                    &uids,
                    |entity, group_change| {
                        clients
                            .get_mut(entity)
                            .and_then(|c| {
                                group_change
                                    .try_map(|e| uids.get(e).copied())
                                    .map(|g| (g, c))
                            })
                            .map(|(g, c)| c.notify(ServerMsg::GroupUpdate(g)));
                    },
                );
            }
        },
        GroupManip::Decline => {
            let mut clients = state.ecs().write_storage::<Client>();
            if let Some(inviter) = clients
                .get_mut(entity)
                .and_then(|c| c.invited_to_group.take())
            {
                // Inform inviter of rejection
                if let Some(client) = clients.get_mut(inviter) {
                    // TODO: say who declined the invite
                    client.notify(ChatType::Meta.server_msg("Invite declined.".to_owned()));
                }
            }
        },
        GroupManip::Leave => {
            let mut clients = state.ecs().write_storage::<Client>();
            let uids = state.ecs().read_storage::<sync::Uid>();
            let mut group_manager = state.ecs().write_resource::<GroupManager>();
            group_manager.leave_group(
                entity,
                &mut state.ecs().write_storage(),
                &state.ecs().read_storage(),
                &uids,
                &state.ecs().entities(),
                &mut |entity, group_change| {
                    clients
                        .get_mut(entity)
                        .and_then(|c| {
                            group_change
                                .try_map(|e| uids.get(e).copied())
                                .map(|g| (g, c))
                        })
                        .map(|(g, c)| c.notify(ServerMsg::GroupUpdate(g)));
                },
            );
        },
        GroupManip::Kick(uid) => {
            let mut clients = state.ecs().write_storage::<Client>();
            let uids = state.ecs().read_storage::<sync::Uid>();
            let alignments = state.ecs().read_storage::<comp::Alignment>();

            let target = match state.ecs().entity_from_uid(uid.into()) {
                Some(t) => t,
                None => {
                    // Inform of failure
                    if let Some(client) = clients.get_mut(entity) {
                        client.notify(
                            ChatType::Meta
                                .server_msg("Kick failed, target does not exist.".to_owned()),
                        );
                    }
                    return;
                },
            };

            // Can't kick pet
            if matches!(alignments.get(target), Some(comp::Alignment::Owned(owner)) if uids.get(target).map_or(true, |u| u != owner))
            {
                if let Some(client) = clients.get_mut(entity) {
                    client.notify(
                        ChatType::Meta.server_msg("Kick failed, you can't kick pets.".to_owned()),
                    );
                }
                return;
            }
            // Can't kick yourself
            if uids.get(entity).map_or(false, |u| *u == uid) {
                if let Some(client) = clients.get_mut(entity) {
                    client.notify(
                        ChatType::Meta
                            .server_msg("Kick failed, you can't kick yourself.".to_owned()),
                    );
                }
                return;
            }

            let mut groups = state.ecs().write_storage::<group::Group>();
            let mut group_manager = state.ecs().write_resource::<GroupManager>();
            // Make sure kicker is the group leader
            match groups
                .get(target)
                .and_then(|group| group_manager.group_info(*group))
            {
                Some(info) if info.leader == entity => {
                    // Remove target from group
                    group_manager.leave_group(
                        target,
                        &mut groups,
                        &state.ecs().read_storage(),
                        &uids,
                        &state.ecs().entities(),
                        &mut |entity, group_change| {
                            clients
                                .get_mut(entity)
                                .and_then(|c| {
                                    group_change
                                        .try_map(|e| uids.get(e).copied())
                                        .map(|g| (g, c))
                                })
                                .map(|(g, c)| c.notify(ServerMsg::GroupUpdate(g)));
                        },
                    );

                    // Tell them the have been kicked
                    if let Some(client) = clients.get_mut(target) {
                        client.notify(
                            ChatType::Meta
                                .server_msg("You were removed from the group.".to_owned()),
                        );
                    }
                    // Tell kicker that they were succesful
                    if let Some(client) = clients.get_mut(entity) {
                        client.notify(ChatType::Meta.server_msg("Player kicked.".to_owned()));
                    }
                },
                Some(_) => {
                    // Inform kicker that they are not the leader
                    if let Some(client) = clients.get_mut(entity) {
                        client.notify(ChatType::Meta.server_msg(
                            "Kick failed: You are not the leader of the target's group.".to_owned(),
                        ));
                    }
                },
                None => {
                    // Inform kicker that the target is not in a group
                    if let Some(client) = clients.get_mut(entity) {
                        client.notify(
                            ChatType::Meta.server_msg(
                                "Kick failed: Your target is not in a group.".to_owned(),
                            ),
                        );
                    }
                },
            }
        },
        GroupManip::AssignLeader(uid) => {
            let mut clients = state.ecs().write_storage::<Client>();
            let uids = state.ecs().read_storage::<sync::Uid>();
            let target = match state.ecs().entity_from_uid(uid.into()) {
                Some(t) => t,
                None => {
                    // Inform of failure
                    if let Some(client) = clients.get_mut(entity) {
                        client.notify(ChatType::Meta.server_msg(
                            "Leadership transfer failed, target does not exist".to_owned(),
                        ));
                    }
                    return;
                },
            };
            let groups = state.ecs().read_storage::<group::Group>();
            let mut group_manager = state.ecs().write_resource::<GroupManager>();
            // Make sure assigner is the group leader
            match groups
                .get(target)
                .and_then(|group| group_manager.group_info(*group))
            {
                Some(info) if info.leader == entity => {
                    // Assign target as group leader
                    group_manager.assign_leader(
                        target,
                        &groups,
                        &state.ecs().entities(),
                        &state.ecs().read_storage(),
                        &uids,
                        |entity, group_change| {
                            clients
                                .get_mut(entity)
                                .and_then(|c| {
                                    group_change
                                        .try_map(|e| uids.get(e).copied())
                                        .map(|g| (g, c))
                                })
                                .map(|(g, c)| c.notify(ServerMsg::GroupUpdate(g)));
                        },
                    );
                    // Tell them they are the leader
                    if let Some(client) = clients.get_mut(target) {
                        client.notify(
                            ChatType::Meta.server_msg("You are the group leader now.".to_owned()),
                        );
                    }
                    // Tell the old leader that the transfer was succesful
                    if let Some(client) = clients.get_mut(target) {
                        client.notify(
                            ChatType::Meta
                                .server_msg("You are no longer the group leader.".to_owned()),
                        );
                    }
                },
                Some(_) => {
                    // Inform transferer that they are not the leader
                    if let Some(client) = clients.get_mut(entity) {
                        client.notify(
                            ChatType::Meta.server_msg(
                                "Transfer failed: You are not the leader of the target's group."
                                    .to_owned(),
                            ),
                        );
                    }
                },
                None => {
                    // Inform transferer that the target is not in a group
                    if let Some(client) = clients.get_mut(entity) {
                        client.notify(ChatType::Meta.server_msg(
                            "Transfer failed: Your target is not in a group.".to_owned(),
                        ));
                    }
                },
            }
        },
    }
}
