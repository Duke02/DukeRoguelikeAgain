use crate::error::DRResult;
use hecs::Entity;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum Event {
    Damage {
        to: Entity,
        from: Entity,
        amount: i32,
    },
}

// pub trait EventData: DeserializeOwned + Debug + Clone + Sized {}
//
// #[derive(Debug, Clone)]
// pub struct Event {
//     pub event_type: EventType,
//     pub payload: Box<dyn EventData>,
// }

/// A static generic event bus implementation that provides immediate event resolution with priority-ordered handler execution.
///
/// > Development Velocity Over Raw Performance
/// This design prioritizes development velocity over raw performance by using immediate event processing rather than deferred execution.
/// While deferred events (like Unity's ECS implementation) can offer better performance in some scenarios, they create awkward system
/// boundaries that significantly slow development velocity. Immediate resolution allows for more intuitive debugging, clearer data flow,
/// and eliminates the complexity of managing event queues and frame-delayed processing.
///
/// > Type-Safe Event Isolation
/// The static generic approach creates a separate event bus instance for each event type T at compile time. This means EventBus{PlayerDiedEvent}
/// is completely separate from EventBus{EnemySpawnedEvent}, providing type safety and eliminating the need for runtime type checking
/// or casting. Each generic instantiation maintains its own handler list and processing logic.
///
/// > Generic Event Hierarchies
/// Events themselves can also be generic, creating even more specific event bus instances. For example, we might have a general
/// ComponentAddedEvent for systems that need to know about any component being added, and a more specific ComponentAddedEvent{HealthComponent}
/// for systems that only care about health components. This creates multiple event buses: EventBus{ComponentAddedEvent} receives all
/// component additions, while EventBus{ComponentAddedEvent{HealthComponent}} only receives health component additions. This pattern
/// allows for both broad and targeted event subscriptions - systems can subscribe to the general event for comprehensive monitoring
/// or to specific generic variants for focused, high-performance processing of only relevant events.
///
/// > Why Static Generics Work Well Here
/// Static generics have one main weakness: enumeration - you cannot iterate over all EventBus{T} instances or discover them at runtime.
/// However, this limitation is irrelevant for event bus scenarios because we never need to enumerate event buses. Each event type
/// knows exactly which bus it belongs to, and handlers subscribe to specific event types they care about. The inability to enumerate
/// actually becomes a feature, providing strong isolation between different event types and preventing accidental cross-contamination
/// or performance overhead from iterating through irrelevant event buses.
///
/// > IMPORTANT: Why Global Static Event Buses __ALWAYS__ Beat Instanced Event Buses
/// The allure of "instanced event buses so the global bus doesn't get cluttered" seems appealing until thoroughly examined.
/// Instanced event buses introduce several critical problems that far outweigh their theoretical benefits:
///
/// 1. **Code Complexity & Lifecycle Management Hell**: Instanced buses require careful registration/unregistration timing.
///    Forgetting to unregister creates memory leaks and ghost handlers. Managing when to create, destroy, and clean up per-object
///    buses adds significant mental overhead and potential for bugs. Global static buses eliminate this entire category of problems.
///
/// 2. **Performance Illusion**: The perceived performance benefit of "only relevant handlers execute" is largely illusory.
///    The overhead of managing separate bus instances, merging priority orders between global and local handlers, and tracking
///    registration state often exceeds the cost of a simple conditional check (like `if (entity.HasComponent{PotionEffect}())`).
///    Modern CPUs handle branch prediction extremely well for such checks.
///
/// 3. **Mental Model Fragmentation**: Global handlers act as "world rules" that are easy to reason about - they consistently
///    apply everywhere and their behavior is predictable. Instanced handlers create context-dependent behavior that's harder
///    to debug and understand. When troubleshooting events, you must consider both global and local handler sets.
///
/// 4. **Premature Optimization**: Most "performance critical" scenarios that seem to justify instanced buses can be better
///    solved through data coalescing. Instead of 30 different healing items each having their own handler, create a single
///    global handler that checks a unified "HealthRegenRate" property. This is faster, cleaner, and more maintainable.
///
/// **The Global Bus "Crowding" Solution**: When the global bus feels crowded with too many specific handlers, this is actually
/// a design smell indicating opportunities for data consolidation. Instead of fragmenting into instanced buses, coalesce
/// related logic into shared properties or components. For example, replace multiple equipment-specific damage handlers with
/// a single handler that reads a "DamageModifiers" component. This approach maintains the simplicity of global rules while
/// achieving the performance benefits that instanced buses promise but rarely deliver.
///
/// (Taken from a friend, thanks Erik!)
pub struct EventBus {
    /// The list of registered event handlers for this specific event type, maintained in priority-ordered execution sequence.
    subscribers: Vec<Box<dyn EventHandler>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Default::default(),
        }
    }

    /// Immediately publishes an event to all registered handlers.
    pub fn publish(&mut self, event: &Event) -> DRResult<()> {
        if self.subscribers.contains()
        if let Some(subscribers) = self.subscribers.get_mut(&event.event_type) {
            for subscriber in subscribers {
                subscriber.on_event(event)?;
            }
        }
        Ok(())
    }

    /// Registers an event handler to receive events of type T, automatically maintaining priority order.
    pub fn subscribe(&mut self, event_type: EventType, handler: Box<dyn EventHandler>) {
        let curr_handlers = self.subscribers.entry(event_type).or_default();
        curr_handlers.push(handler);
        // This might be more efficient if we just find where to put the new handler.
        curr_handlers.sort_by(|one, two| one.get_priority().cmp(&two.get_priority()));
    }
}

/// A priority-based event handler that wraps an action delegate and provides automatic registration/unregistration
/// with the EventBus system. Handlers with lower priority values are executed first.
///
/// Design Rationale:
/// This implementation uses a priority-based execution order with action delegates rather than a dependency-based
/// system with dedicated handler classes for several key reasons:
///
/// 1. Object-Oriented Flexibility: Unlike dependency-based systems that work well for system-level programming,
///    this approach accommodates object instances naturally. Objects can easily create handlers for their methods
///    without requiring singleton patterns or complex instantiation logic.
///
/// 2. Simplified Mental Model: Priority numbers (0, 1, 2...) are more straightforward to reason about than
///    declaring and managing dependencies between handler types. Developers can quickly understand execution
///    order without mapping dependency graphs.
///
/// 3. Naming Convenience: Methods within existing classes can serve as event handlers directly, eliminating
///    the need to create and name dedicated handler classes. This reduces boilerplate and keeps related
///    logic co-located.
///
/// 4. Reduced Complexity: Avoids reflection-based auto-registration and singleton management that would be
///    required for abstract class-based handlers, making the system more predictable and debuggable.
///
/// While dependency-based ordering has theoretical appeal, the practical benefits of this simpler approach
/// outweigh the elegance of dependency declarations for most use cases. I was really attached to the
/// dependency-based solution, but after careful consideration, I believe this design will be more
/// maintainable and easier to work with in the long run.
///
/// (This was also taken from Erik, thanks King!)
pub trait EventHandler {
    fn on_event(&mut self, event: &Event) -> DRResult<()>;
    fn get_priority(&self) -> u32;
}
