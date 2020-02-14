use crate::{
    comp::{ActionState, CharacterState, Energy, EnergySource, HealthSource, MovementState, Stats},
    event::{EventBus, ServerEvent},
    state::DeltaTime,
};
use specs::{Entities, Join, Read, ReadStorage, System, WriteStorage};

const ENERGY_REGEN_ACCEL: f32 = 20.0;

/// This system kills players, levels them up, and regenerates energy.
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        Read<'a, DeltaTime>,
        Read<'a, EventBus<ServerEvent>>,
        ReadStorage<'a, CharacterState>,
        WriteStorage<'a, Stats>,
        WriteStorage<'a, Energy>,
    );

    fn run(
        &mut self,
        (entities, dt, server_event_bus, character_states, mut stats,mut energies): Self::SystemData,
    ) {
        let mut server_event_emitter = server_event_bus.emitter();

        // Increment last change timer
        stats.set_event_emission(false); // avoid unnecessary syncing
        for stat in (&mut stats).join() {
            stat.health.last_change.0 += f64::from(dt.0);
        }
        stats.set_event_emission(true);

        // Mutates all stats every tick causing the server to resend this component for
        // every entity every tick
        for (entity, character_state, mut stats, mut energy) in (
            &entities,
            &character_states,
            &mut stats.restrict_mut(),
            &mut energies.restrict_mut(),
        )
            .join()
        {
            let (set_dead, level_up) = {
                let stat = stats.get_unchecked();
                (
                    stat.should_die() && !stat.is_dead,
                    stat.exp.current() >= stat.exp.maximum(),
                )
            };

            if set_dead {
                let stat = stats.get_mut_unchecked();
                server_event_emitter.emit(ServerEvent::Destroy {
                    entity,
                    cause: stat.health.last_change.1.cause,
                });

                stat.is_dead = true;
            }

            if level_up {
                let stat = stats.get_mut_unchecked();
                while stat.exp.current() >= stat.exp.maximum() {
                    stat.exp.change_by(-(stat.exp.maximum() as i64));
                    stat.exp.change_maximum_by(25);
                    stat.level.change_by(1);
                }

                stat.update_max_hp();
                stat.health
                    .set_to(stat.health.maximum(), HealthSource::LevelUp);
            }

            // Accelerate recharging energy if not wielding.
            match character_state.action {
                ActionState::Idle => {
                    if {
                        let energy = energy.get_unchecked();
                        energy.current() < energy.maximum()
                    } {
                        let mut energy = energy.get_mut_unchecked();
                        // Have to account for Calc I differential equations due to acceleration
                        energy.change_by(
                            (energy.regen_rate * dt.0 + ENERGY_REGEN_ACCEL * dt.0.powf(2.0) / 2.0)
                                as i32,
                            EnergySource::Regen,
                        );
                        energy.regen_rate += ENERGY_REGEN_ACCEL * dt.0;
                    }
                },
                // All other states do not regen and set the rate back to zero.
                _ => {
                    if energy.get_unchecked().regen_rate != 0.0 {
                        energy.get_mut_unchecked().regen_rate = 0.0
                    }
                },
            }

            match character_state.movement {
                MovementState::Climb => {
                    if energy.get_unchecked().regen_rate != 0.0 {
                        energy.get_mut_unchecked().regen_rate = 0.0
                    }
                },
                MovementState::Glide => {
                    if energy.get_unchecked().regen_rate != 0.0 {
                        energy.get_mut_unchecked().regen_rate = 0.0
                    }
                },
                MovementState::Swim => {
                    if energy.get_unchecked().regen_rate != 0.0 {
                        energy.get_mut_unchecked().regen_rate = 0.0
                    }
                },
                _ => {
                    continue;
                },
            }
        }
    }
}
