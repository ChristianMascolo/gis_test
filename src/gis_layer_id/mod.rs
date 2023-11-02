#![warn(
clippy::unwrap_used,
clippy::cast_lossless,
clippy::unimplemented,
clippy::indexing_slicing,
clippy::expect_used
)]

#[derive(
Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, bevy::ecs::component::Component,
)]
pub struct LayerId(i32);

impl Default for LayerId {
    fn default() -> Self {
        Self::new()
    }
}

impl LayerId {
    pub fn new() -> Self {
        LayerId(0)
    }

    pub fn get_id(&self) -> i32 {
        self.0
    }
}

pub fn new_id(last: i32) -> i32 {
    last+1
}
