use crate::error::{DRError, DRResult};
use crate::events::EventBusManager;
use crate::events::{DeadEntity, EventBus, EventHandler};
use crate::models::ai::{Action, Ai, Vision};
use crate::models::input::InputState;
use crate::models::stats::{Damage, Health};
use crate::models::{Player, Position};
use crate::{CONSOLE_HEIGHT, CONSOLE_WIDTH};
use doryen_rs::DoryenApi;
use hecs::{Entity, PreparedQuery, Ref, With, World};
use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::Arc;
use tracing::{event, warn};

fn get_entity_locations(world: &World) -> HashMap<Position, Entity> {
    let positions = world
        .query::<&Position>()
        .view()
        .into_iter()
        .map(|(id, pos)| (pos.clone(), id))
        .collect::<HashMap<Position, Entity>>();
    tracing::trace!(?positions, "get_entity_locations");
    positions
}

pub trait SystemFunc {
    fn call(
        &mut self,
        world: &mut World,
        api: &mut dyn DoryenApi,
        event_bus_manager: &mut EventBusManager,
    ) -> DRResult<()>;

    fn init(&mut self, world: &mut World, event_bus_manager: &mut EventBusManager) {}

    fn get_name(&self) -> String;
}

pub struct InputSystem {
    input_state_entity_id: Option<Entity>,
}

impl Default for InputSystem {
    fn default() -> InputSystem {
        InputSystem {
            input_state_entity_id: None,
        }
    }
}

impl SystemFunc for InputSystem {
    fn call(
        &mut self,
        world: &mut World,
        api: &mut dyn DoryenApi,
        event_bus_manager: &mut EventBusManager,
    ) -> DRResult<()> {
        tracing::trace!("InputSystem::call");
        // let world = Arc::new(RefCell::new(world));
        // let mut binding = (*world).borrow_mut();
        let entity_locations = get_entity_locations(world);
        let player_input_id = self
            .input_state_entity_id
            .ok_or(DRError::MissingEntity("player".to_string()))?;
        let player = match world.entity(player_input_id) {
            Ok(player) => player,
            Err(e) => {
                tracing::warn!(
                    "Cannot find player! Got error {e} Was it added? Assuming game over."
                );
                return Err(DRError::GameOver);
            }
        };
        let input = api.input();

        // let mut had_input = false;
        let mut player_pos = world.get::<&mut Position>(player.entity())?;
        let mut next_position = None;

        if input.key("ArrowLeft") {
            next_position = Some(player_pos.new_from_dx_dy(-1, 0));
            // player_pos.x = (player_pos.x - 1).max(1);
        } else if input.key("ArrowRight") {
            next_position = Some(player_pos.new_from_dx_dy(1, 0));
            // player_pos.x = (player_pos.x + 1).min((CONSOLE_WIDTH as i32 - 2) as isize);
        } else if input.key("ArrowUp") {
            next_position = Some(player_pos.new_from_dx_dy(0, -1));
            // player_pos.y = (player_pos.y - 1).max(1);
        } else if input.key("ArrowDown") {
            next_position = Some(player_pos.new_from_dx_dy(0, 1));
            // player_pos.y = (player_pos.y + 1).min((CONSOLE_HEIGHT as i32 - 2) as isize);
        }

        // let input_state_query = world.query()
        let mut input_state = world.get::<&mut InputState>(
            self.input_state_entity_id
                .expect("Input System was not initialized!"),
        )?;
        input_state.was_input_handled_this_frame = false;
        if let Some(next_position) = next_position {
            if next_position.is_within_console_bounds()
                && !entity_locations.contains_key(&next_position)
            {
                tracing::debug!("Flipping the input state!");
                input_state.was_input_handled_this_frame = true;

                player_pos.x = next_position.x;
                player_pos.y = next_position.y;
                drop(player_pos);
            } else if let Some(entity) = entity_locations.get(&next_position) {
                tracing::debug!("Attacking entity {entity:?}");
                input_state.was_input_handled_this_frame = true;
                event_bus_manager.enqueue(Damage {
                    from: player_input_id,
                    to: entity.clone(),
                    damage: 2,
                });
            }
        }

        Ok(())
    }

    fn init(&mut self, world: &mut World, event_bus_manager: &mut EventBusManager) {
        self.input_state_entity_id = Some(
            world
                .query::<&InputState>()
                .iter()
                .next()
                .expect("InputState not found in world.")
                .0,
        );
        // self.input_state_entity_id = Some(world.spawn((InputState::default(),)));
    }

    fn get_name(&self) -> String {
        "InputSystem".to_string()
    }
}

pub struct AiSystem {
    health_query: PreparedQuery<With<&'static Position, &'static Health>>,
    ai_query: PreparedQuery<(
        &'static mut Ai,
        &'static mut Position,
        &'static Health,
        &'static Vision,
    )>,
    player_entity_id: Option<Entity>,
}

impl AiSystem {
    pub fn new() -> AiSystem {
        Self {
            health_query: PreparedQuery::new(),
            ai_query: PreparedQuery::new(),
            player_entity_id: None,
        }
    }

    fn get_entity_locs(&mut self, world: &mut World) -> HashSet<Position> {
        get_entity_locations(world)
            .keys()
            .into_iter()
            .map(|pos| pos.clone())
            .collect()
    }

    fn was_input_handled_this_frame(&self, world: &World) -> bool {
        let input_state = world
            .get::<&InputState>(self.player_entity_id.unwrap())
            .expect("Could not get input state!");
        input_state.was_input_handled_this_frame
        // let mut binding = world.query::<&InputState>();
        // let (_entity, input_state) = binding.into_iter().next().unwrap();
        // let was_input_handled_this_frame = input_state.was_input_handled_this_frame;
        // tracing::trace!(
        //     ?was_input_handled_this_frame,
        //     "was_input_handled_this_frame"
        // );
        // was_input_handled_this_frame
    }

    // fn get_player_pos_health<'a>(
    //     &self,
    //     world: &'a Arc<RefCell<&'a mut World>>,
    // ) -> Option<(Position, RefMut<'a, Health>)> {
    //     Some((position, health))
    // }

    // fn get_ais(&self)
}

impl SystemFunc for AiSystem {
    fn call(
        &mut self,
        world: &mut World,
        api: &mut dyn DoryenApi,
        event_bus_manager: &mut EventBusManager,
    ) -> DRResult<()> {
        if !self.was_input_handled_this_frame(&world) {
            tracing::trace!("Player didn't do any input so skipping AI...");
            return Ok(());
        }
        tracing::debug!("AiSystem::call");
        // player_pos: &Position,
        //         my_position: &Position,
        //         my_health: &Health,
        //         my_vision: &Vision,

        // let world = Arc::new(RefCell::new(world));

        let has_entity = self.get_entity_locs(world);

        let player_id = self
            .player_entity_id
            .ok_or(DRError::MissingEntity("player".to_string()))?;

        let player = world.entity(player_id.clone())?;

        tracing::debug!("Getting player pos...");

        let player_pos = player
            .get::<&Position>()
            .ok_or(DRError::ComponentMissing("Position".to_string()))?
            .deref()
            .clone();
        // let mut player_health = world.get::<&mut Health>(
        //     self.player_entity_id
        //         .ok_or(DRError::MissingEntity("player".to_string()))?,
        // )?;

        // let (player_pos, mut player_health) = self
        //     .get_player_pos_health(&world)
        //     .ok_or(DRError::ComponentMissing("Position/Health".to_string()))?;

        let binding = self.ai_query.borrow_mut();
        let ai_query = binding.query_mut(world);
        tracing::info!("Processing AIs...");
        for (id, (ai, ai_pos, ai_health, ai_vision)) in ai_query {
            let action = ai.get_next_action(&player_pos, ai_pos, ai_health, ai_vision);
            tracing::debug!("Entity with ID {id:?} will do action {action:?}");
            match action {
                Action::GoTo(new_pos) => {
                    // TODO: Add bounds/occupancy checking.
                    let next_pos = ai_pos.go_towards(&new_pos);
                    if !has_entity.contains(&next_pos) {
                        let Position { x, y } = next_pos;
                        ai_pos.x = x;
                        ai_pos.y = y;
                    }
                }
                Action::Wait => {} // Do Nothing.
                Action::Attack(pos_to_attack) => {
                    if has_entity.contains(&pos_to_attack) {
                        tracing::debug!(
                            "Entity with ID {id:?} attacked the entity at {pos_to_attack:?}"
                        );
                        event_bus_manager.enqueue(Damage {
                            from: id,
                            to: player_id.clone(),
                            damage: 1,
                        });
                    } else {
                        tracing::debug!(
                            "Entity with ID {id:?} tried to attack the empty air at {pos_to_attack:?}."
                        )
                    }
                }
            }
        }
        Ok(())
    }
    fn init(&mut self, world: &mut World, event_bus_manager: &mut EventBusManager) {
        tracing::debug!("AiSystem::init");
        self.player_entity_id = Some(
            world
                .query::<&Player>()
                .iter()
                .next()
                .expect("Have not initialized player yet.")
                .0,
        );
    }

    fn get_name(&self) -> String {
        "AISystem".to_string()
    }
}

/// BRING OUT YOUR DEAD!!
pub struct DeadCollector {
    // dead_finder: PreparedQuery<&'static Health>,
}

impl Default for DeadCollector {
    fn default() -> Self {
        Self {}
    }
}

impl EventHandler<DeadEntity> for DeadCollector {
    fn handle(&self, event: &mut DeadEntity, world: &mut World) {
        match world.despawn(event.entity) {
            Ok(()) => (),
            Err(e) => {
                tracing::warn!("Could not despawn supposedly dead entity due to error {e}");
                ()
            }
        };
    }
}

// impl SystemFunc for DeadCollector {
//     fn call(
//         &mut self,
//         world: &mut World,
//         api: &mut dyn DoryenApi,
//         event_bus_manager: &mut EventBusManager,
//     ) -> DRResult<()> {
//         let ones_to_remove: Vec<_> = self
//             .dead_finder
//             .query(world)
//             .iter()
//             .filter(|(_, health)| health.current_health <= 0)
//             .map(|(id, _health)| id)
//             .collect();
//
//         for id in ones_to_remove {
//             world.despawn(id)?;
//         }
//         Ok(())
//     }
//     fn init(&mut self, world: &mut World, event_bus_manager: &mut EventBusManager) {
//         self.dead_finder = PreparedQuery::new();
//     }
//
//     fn get_name(&self) -> String {
//         "DeadCollector".to_string()
//     }
// }

/// Deletes dead AIs and spawns new ones as needed.
struct AiHandlerSystem;

impl SystemFunc for AiHandlerSystem {
    fn call(
        &mut self,
        world: &mut World,
        api: &mut dyn DoryenApi,
        event_bus_manager: &mut EventBusManager,
    ) -> DRResult<()> {
        todo!()
    }

    fn get_name(&self) -> String {
        "AiHandlerSystem".to_string()
    }
}

#[derive(Default)]
pub struct DamageSystem {
    damage_query: PreparedQuery<&'static Damage>,
}

impl SystemFunc for DamageSystem {
    fn call(
        &mut self,
        world: &mut World,
        api: &mut dyn DoryenApi,
        event_bus_manager: &mut EventBusManager,
    ) -> DRResult<()> {
        // Get all entities that need damage applied to them
        // Then remove the health from them.
        for (_, damage) in self.damage_query.query(&world).iter() {
            let mut damaged_entity = world.get::<&mut Health>(damage.to)?;
            damaged_entity.current_health -= damage.damage;
        }

        Ok(())
    }
    fn init(&mut self, world: &mut World, event_bus_manager: &mut EventBusManager) {
        self.damage_query = PreparedQuery::new();
    }

    fn get_name(&self) -> String {
        "DamageSystem".to_string()
    }
}
