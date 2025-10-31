mod all_events;
mod event_bus;
mod event_bus_manager;

pub use crate::events::all_events::*;
pub use crate::events::event_bus::EventBus;
pub use crate::events::event_bus_manager::EventBusManager;
use std::any::Any;
use hecs::World;

pub trait Event: Any + Send + Sync + 'static {}
impl<T: Any + Send + Sync + 'static> Event for T {}

pub trait EventHandler<T: Event>: Send + Sync {
    fn handle(&self, event: &mut T, world: &mut World);
}
