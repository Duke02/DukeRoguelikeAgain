use crate::events::Event;
use hecs::Entity;

#[derive(Debug, Clone)]
pub struct DeadEntity {
    pub entity: Entity,
}
