use common::{
    comp::{HealthSource, Object, PhysicsState, Pos},
    event::{EventBus, ServerEvent},
    state::DeltaTime,
};
use specs::{Entities, Join, Read, ReadStorage, System, WriteStorage};

/// This system is responsible for handling misc object behaviours
pub struct Sys;
impl<'a> System<'a> for Sys {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        Read<'a, DeltaTime>,
        Read<'a, EventBus<ServerEvent>>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, PhysicsState>,
        WriteStorage<'a, Object>,
    );

    fn run(
        &mut self,
        (entities, _dt, server_bus, positions, physics_states, mut objects): Self::SystemData,
    ) {
        let mut server_emitter = server_bus.emitter();

        // Objects
        for (entity, pos, physics, object) in
            (&entities, &positions, &physics_states, &mut objects).join()
        {
            match object {
                Object::Bomb { owner } => {
                    if physics.on_surface().is_some() {
                        server_emitter.emit(ServerEvent::Destroy {
                            entity,
                            cause: HealthSource::Suicide,
                        });
                        server_emitter.emit(ServerEvent::Explosion {
                            pos: pos.0,
                            power: 4.0,
                            owner: *owner,
                            friendly_damage: true,
                        });
                    }
                },
            }
        }
    }
}
