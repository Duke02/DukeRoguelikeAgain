use crate::error::{DRError, DRResult};
use crate::models::ai::{Action, Ai, Vision};
use crate::models::input::InputState;
use crate::models::{Health, Player, Position};
use crate::{CONSOLE_HEIGHT, CONSOLE_WIDTH};
use doryen_rs::DoryenApi;
use hecs::{Entity, PreparedQuery, RefMut, With, World};
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;

pub trait SystemFunc {
    fn call(&mut self, world: &mut World, api: &mut dyn DoryenApi) -> DRResult<()>;

    fn init(&mut self, world: &mut World) {}
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
    fn call(&mut self, world: &mut World, api: &mut dyn DoryenApi) -> DRResult<()> {
        let world = Arc::new(RefCell::new(world));
        let mut binding = (*world).borrow_mut();
        let mut player_pos_query = binding.query_mut::<(&mut Position, &Player)>();

        let mut binding = (*api).borrow_mut();
        let input = binding.input();

        let mut had_input = false;

        for (_id, (player_pos, _player)) in player_pos_query.view().iter_mut() {
            if input.key("ArrowLeft") {
                player_pos.x = (player_pos.x - 1).max(1);
                had_input = true;
            } else if input.key("ArrowRight") {
                player_pos.x = (player_pos.x + 1).min((CONSOLE_WIDTH as i32 - 2) as isize);
                had_input = true;
            }
            if input.key("ArrowUp") {
                player_pos.y = (player_pos.y - 1).max(1);
                had_input = true;
            } else if input.key("ArrowDown") {
                player_pos.y = (player_pos.y + 1).min((CONSOLE_HEIGHT as i32 - 2) as isize);
                had_input = true;
            }
            // We should only have a single player instance in the entire thing.
            break;
        }
        drop(player_pos_query);
        // let input_state_query = world.query()
        let binding = world.borrow();
        let mut input_state = binding.get::<&mut InputState>(
            self.input_state_entity_id
                .expect("Input System was not initialized!"),
        )?;
        input_state.was_input_handled_this_frame = had_input;
        Ok(())
    }

    fn init(&mut self, world: &mut World) {
        self.input_state_entity_id = Some(world.spawn((InputState::default(),)));
    }
}

pub struct AiSystem {
    health_query: RefCell<PreparedQuery<With<&'static Position, &'static Health>>>,
    ai_query: RefCell<
        PreparedQuery<(
            &'static mut Ai,
            &'static mut Position,
            &'static Health,
            &'static Vision,
        )>,
    >,
    player_entity_id: Option<Entity>,
}

impl AiSystem {
    pub fn new() -> AiSystem {
        Self {
            health_query: RefCell::new(PreparedQuery::new()),
            ai_query: RefCell::new(PreparedQuery::new()),
            player_entity_id: None,
        }
    }

    fn get_entity_locs(&self, world: &Arc<RefCell<&mut World>>) -> HashSet<Position> {
        let binding = world.borrow();
        self.health_query
            .borrow_mut()
            .query(&**binding)
            .view()
            .into_iter()
            .map(|(_id, pos)| pos.clone())
            .collect::<HashSet<Position>>()
    }

    fn was_input_handled_this_frame(&self, world: &World) -> bool {
        let mut binding = world.query::<&InputState>();
        let (_entity, input_state) = binding.into_iter().next().unwrap();
        input_state.was_input_handled_this_frame
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
    fn call(&mut self, world: &mut World, api: &mut dyn DoryenApi) -> DRResult<()> {
        if !self.was_input_handled_this_frame(&world) {
            return Ok(());
        }

        // player_pos: &Position,
        //         my_position: &Position,
        //         my_health: &Health,
        //         my_vision: &Vision,

        let world = Arc::new(RefCell::new(world));

        let has_entity = self.get_entity_locs(&world);

        let player_pos = world
            .borrow()
            .get::<&Position>(
                self.player_entity_id
                    .ok_or(DRError::MissingEntity("player".to_string()))?,
            )?
            .deref()
            .clone();
        let binding = world.borrow();
        let mut player_health = binding.get::<&mut Health>(
            self.player_entity_id
                .ok_or(DRError::MissingEntity("player".to_string()))?,
        )?;

        // let (player_pos, mut player_health) = self
        //     .get_player_pos_health(&world)
        //     .ok_or(DRError::ComponentMissing("Position/Health".to_string()))?;

        let mut binding = self.ai_query.borrow_mut();
        let mut binding2 = (*world).borrow_mut();
        let ai_query = binding.query_mut(&mut binding2);

        for (id, (ai, ai_pos, ai_health, ai_vision)) in ai_query {
            let action = ai.get_next_action(&player_pos, ai_pos, ai_health, ai_vision);
            println!("Entity with ID {id:?} will do action {action:?}");
            match action {
                Action::GoTo(new_pos) => {
                    // TODO: Add bounds/occupancy checking.
                    let Position { x, y } = ai_pos.go_towards(&new_pos);
                    ai_pos.x = x;
                    ai_pos.y = y
                }
                Action::Wait => {} // Do Nothing.
                Action::Attack(pos_to_attack) => {
                    if has_entity.contains(&pos_to_attack) {
                        println!("Entity with ID {id:?} attacked the entity at {pos_to_attack:?}");
                    } else {
                        println!(
                            "Entity with ID {id:?} tried to attack the empty air at {pos_to_attack:?} and looked like a dumbass."
                        )
                    }
                }
            }
        }
        Ok(())
    }
    fn init(&mut self, world: &mut World) {
        self.player_entity_id = Some(
            world
                .query::<&Player>()
                .iter()
                .next()
                .expect("Have not initialized player yet.")
                .0,
        );
    }
}

/// Deletes dead AIs and spawns new ones as needed.
struct AiHandlerSystem;

impl SystemFunc for AiHandlerSystem {
    fn call(&mut self, world: &mut World, api: &mut dyn DoryenApi) -> DRResult<()> {
        todo!()
    }
}
