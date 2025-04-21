//
// reviewed: 2025-04-21
//
use crate::lib::fixed_size_string::FixedSizeString;
use alloc::vec::Vec;

pub type LocationId = usize;
pub type LinkId = usize;
pub type ObjectId = usize;
pub type EntityId = usize;
pub type Name = FixedSizeString<32>;
pub type Note = FixedSizeString<64>;
pub type EntityMessage = FixedSizeString<128>;

pub struct World {
    pub objects: Vec<Object>,
    pub entities: Vec<Entity>,
    pub locations: Vec<Location>,
    pub links: Vec<Link>,
}

pub struct Location {
    pub name: Name,
    pub note: Note,
    pub links: Vec<LocationLink>,
    pub objects: Vec<ObjectId>,
    pub entities: Vec<EntityId>,
}

pub struct LocationLink {
    pub link: LinkId,
    pub location: LocationId,
}

pub struct Link {
    pub name: Name,
}

pub struct Object {
    pub name: Name,
}

pub struct Entity {
    pub name: Name,
    pub location: LocationId,
    pub objects: Vec<ObjectId>,
    pub messages: Vec<EntityMessage>,
}
