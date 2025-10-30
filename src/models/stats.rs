use std::any::TypeId;
use std::ptr::NonNull;
use hecs::{Bundle, Entity, MissingComponent, TypeInfo};

#[derive(Debug)]
pub struct Health {
    pub total_health: u32,
    pub current_health: i32,
}

impl Health {
    pub fn new(health: u32) -> Health {
        Health {
            total_health: health,
            current_health: health as i32,
        }
    }
    pub fn get_ratio(&self) -> f32 {
        let ratio = self.current_health as f32 / self.total_health as f32;
        tracing::trace!(?ratio, ?self, "get_ratio");
        ratio
    }
}

#[derive(Debug)]
pub struct Damage {
    pub from: Entity,
    pub to: Entity,
    pub damage: i32,
}

// impl Bundle for Damage {
//     fn with_static_ids<T>(f: impl FnOnce(&[TypeId]) -> T) -> T {
//         todo!()
//     }
//
//     fn with_static_type_info<T>(f: impl FnOnce(&[TypeInfo]) -> T) -> T {
//         todo!()
//     }
//
//     unsafe fn get(f: impl FnMut(TypeInfo) -> Option<NonNull<u8>>) -> Result<Self, MissingComponent>
//     where
//         Self: Sized
//     {
//         todo!()
//     }
// }
