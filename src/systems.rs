use crate::models::ai::{Action, Ai, Vision};
use crate::models::input::InputState;
use crate::models::{Health, Player, Position};
use crate::{CONSOLE_HEIGHT, CONSOLE_WIDTH};
use doryen_rs::DoryenApi;
use hecs::{Entity, World};
use std::collections::HashSet;
use std::error::Error;
use std::ops::Deref;

pub trait SystemFunc {
    fn call(&mut self, world: &mut World, api: &mut dyn DoryenApi) -> Result<(), Box<dyn Error>>;

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
    fn call(&mut self, world: &mut World, api: &mut dyn DoryenApi) -> Result<(), Box<dyn Error>> {
        let mut player_pos_query = world.query_mut::<(&mut Position, &Player)>();

        let input = api.input();

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
        let mut input_state = world.get::<&mut InputState>(
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

pub struct AiSystem;

impl SystemFunc for AiSystem {
    fn call(&mut self, world: &mut World, api: &mut dyn DoryenApi) -> Result<(), Box<dyn Error>> {
        let mut binding = world
            .query::<&InputState>();
        let (_entity, input_state) = binding
            .into_iter()
            .next()
            .unwrap()
            ;
        if !input_state.was_input_handled_this_frame {
            return Ok(());
        }
        let mut player_pos: Option<&Position> = None;
        let mut binding = world.query::<(&Position, &Player)>();
        for (_id, (player_pos_w, _player)) in binding.iter() {
            player_pos = Some(player_pos_w);
            break;
        }
        let player_pos = player_pos.expect("Cannot find player position.");
        // player_pos: &Position,
        //         my_position: &Position,
        //         my_health: &Health,
        //         my_vision: &Vision,

        let mut health_pos_query = world.query::<(&Position, &Health)>();
        let mut has_entity = health_pos_query
            .view()
            .into_iter()
            .map(|(_id, (pos, _health))| pos.clone())
            .collect::<HashSet<Position>>();
        has_entity.insert(player_pos.clone());
        drop(health_pos_query);

        let mut ai_query = world.query::<(&mut Ai, &mut Position, &Health, &Vision)>();

        for (id, (ai, ai_pos, ai_health, ai_vision)) in ai_query.view().iter_mut() {
            let action = ai.get_next_action(player_pos, ai_pos, ai_health, ai_vision);
            match action {
                Action::GoTo(new_pos) => *ai_pos = ai_pos.go_towards(&new_pos, 1),
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
}
