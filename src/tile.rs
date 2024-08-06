
pub use tile_full::Tile as Tile;
pub use tile_id::TileId as TileId;

mod tile_full {
    use crate::physics2::CollisionResult;

    #[derive(Default, Clone, Copy, PartialEq)]
    pub enum Breakable {
        WithTime(f32),
        #[default]
        Instantly,
        Indestructable
    }
    
    #[derive(Clone, Copy, PartialEq, Eq, Default)]
    pub enum TilePhysicality {
        #[default]
        Solid,
        JumpThrough,
        Empty,
    }
    
    #[derive(Clone, Copy, PartialEq, Default)]
    pub struct Tile {
        pub sprite: Option<u32>,
        pub breakable: Breakable,
        pub name: &'static str,
        pub physicality: TilePhysicality,
    }
    
    pub(super) const EMPTY_TILE: Tile = Tile {
        sprite: None,
        breakable: Breakable::Indestructable,
        name: "EMPTY",
        physicality: TilePhysicality::Empty
    };
    
    impl Tile {
        pub fn collision_result(&self) -> CollisionResult {
            use TilePhysicality as TP;
            use CollisionResult as CR;
            
            match self.physicality {
                TP::Solid => CR::Solid,
                TP::JumpThrough => CR::JumpThrough,
                TP::Empty => CR::Empty
            }
        }
    }
}

mod tile_id {
    use serde::{Deserialize, Serialize};

    use crate::tile::tile_full::{Tile, EMPTY_TILE};

    use super::tile_full::{Breakable, TilePhysicality};
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
    pub enum TileId {
        #[default]
        Air,
        Dirt,
        WoodPlanks
    }
    
    impl TileId {
        pub const fn val(self) -> &'static Tile {
            use TileId::*;
            use Breakable::*;
            use TilePhysicality::*;
            type T = Tile;
            
            match self {
                Air => &T {
                    name: "Air",
                    ..EMPTY_TILE
                },
                Dirt => &T {
                    breakable: WithTime(1.0),
                    name: "Dirt",
                    physicality: Solid,
                    sprite: Some(9)
                },
                WoodPlanks => &T {
                    breakable: WithTime(2.0),
                    name: "Wood",
                    physicality: Solid,
                    sprite: Some(33)
                }
            }
        }
    }
}