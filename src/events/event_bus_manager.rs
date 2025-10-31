use crate::events::{Event, EventBus, EventHandler};
use hecs::World;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct EventBusManager {
    buses: Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
    // world: Mutex<Arc<World>>,
    queued_events: Mutex<Vec<Box<dyn Event>>>,
}

impl EventBusManager {
    pub fn new() -> Self {
        Self {
            buses: Mutex::new(HashMap::new()),
            queued_events: Mutex::new(Vec::new()),
        }
    }

    /// Get or create a bus for the given event type
    fn get_or_create_bus<T: Event>(&self) -> Arc<Mutex<EventBus<T>>> {
        let mut map = self
            .buses
            .lock()
            .expect("Lock could not be established to get/create event bus.");
        if let Some(bus_any) = map.get(&TypeId::of::<T>()) {
            // Downcast back to the concrete EventBus<T>
            bus_any
                .downcast_ref::<Arc<Mutex<EventBus<T>>>>()
                .expect("Could not downcast event bus")
                .to_owned()
        } else {
            // Create a new bus and store it
            let new_bus = Arc::new(Mutex::new(EventBus::<T>::new()));
            map.insert(TypeId::of::<T>(), Box::new(new_bus.clone()));
            new_bus
        }
    }

    /// Subscribe to an event type
    pub fn subscribe<T: Event>(&self, handler: Arc<dyn EventHandler<T>>) {
        let bus = self.get_or_create_bus::<T>();
        bus.lock()
            .expect("Could not establish lock to subscribe to event bus.")
            .subscribe(handler);
    }

    /// Publish an event of any type
    fn post<T: Event>(&self, mut event: T, world: &mut World) {
        let bus = self.get_or_create_bus::<T>();
        let bus_locked = bus
            .lock()
            .expect("Could not establish lock to post to event bus.");
        // let mut binding = self
        //     .world
        //     .lock()
        //     .expect("Could not establish lock to world during post.")
        //     .clone();
        // let world_locked = binding.borrow_mut();
        bus_locked.publish(&mut event, world);
    }
    pub fn enqueue<T: Event>(&self, event: T) {
        self.queued_events
            .lock()
            .expect("Tried to acquire lock for queued events to enqueue an event.")
            .push(Box::new(event));
    }

    pub fn dispatch_all(&self, world: &mut hecs::World) {
        let mut queue = self.queued_events.lock().unwrap();
        for event_box in queue.drain(..) {
            // We need to downcast by TypeId like before
            let type_id = (*event_box).type_id();
            if let Some(bus_any) = self.buses.lock().unwrap().get(&type_id) {
                // Downcast the event back to the right type
                // dispatch_event_to_bus(bus_any, event_box, world);
                self.post(event_box, world);
            }
        }
    }
}
