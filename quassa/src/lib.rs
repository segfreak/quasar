pub mod ir;
pub mod opt;
pub mod target;
pub mod verify;

pub mod prelude {
    #[cfg(feature = "hashbrown")]
    pub use hashbrown::HashMap;
    #[cfg(not(feature = "hashbrown"))]
    pub use std::collections::HashMap;

    #[cfg(feature = "hashbrown")]
    pub use hashbrown::HashSet;
    #[cfg(not(feature = "hashbrown"))]
    pub use std::collections::HashSet;
}
