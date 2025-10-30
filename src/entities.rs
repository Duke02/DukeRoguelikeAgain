use crate::models::ai::{Ai, Vision};
use crate::models::stats::Health;
use crate::models::{Position, Renderable};
use hecs::World;

pub fn spawn_goblin(
    world: &mut World,
    num_goblins: usize,
    (min_health, max_health): (u32, u32),
    (map_width, map_height): (usize, usize),
) {
    tracing::debug!(?num_goblins, ?min_health, ?max_health, "spawn_goblin");
    let goblins: Vec<_> = (0..num_goblins)
        .map(|_| {
            let ai = Ai::default();
            let vision = Vision::new(6);
            let pos = Position::new(
                (rand::random::<u32>() as usize % map_width) as isize + 1,
                (rand::random::<u32>() as usize % map_height) as isize + 1,
            );
            let health =
                Health::new(rand::random::<u32>() % (max_health - min_health) + min_health);
            let renderable = Renderable {
                glyph: 'G',
                color: (92, 255, 92, 255),
            };
            (ai, pos, health, vision, renderable)
        })
        .collect();
    tracing::trace!(?goblins);
    world.spawn_batch(goblins);
}
