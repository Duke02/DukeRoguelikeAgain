use crate::events::{Event, EventHandler};
use hecs::World;
use std::sync::Arc;

pub struct EventBus<T: Event> {
    handlers: Vec<Arc<dyn EventHandler<T>>>,
}

impl<T: Event> EventBus<T> {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, handler: Arc<dyn EventHandler<T>>) {
        self.handlers.push(handler);
    }

    pub fn publish(&self, event: &mut T, world: &mut World) {
        for handler in &self.handlers {
            handler.handle(event, world);
        }
    }
}
