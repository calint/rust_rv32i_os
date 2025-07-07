//
// reviewed: 2025-04-21
//
use crate::lib::fixed_size_string::FixedSizeString;
use alloc::vec::Vec;

pub type LocationId = usize;
pub type LinkNameId = usize;
pub type ObjectId = usize;
pub type EntityId = usize;
pub type Name = FixedSizeString<32>;
pub type Note = FixedSizeString<64>;
pub type Message = FixedSizeString<128>;

pub struct World {
    pub objects: Vec<Object>,
    pub entities: Vec<Entity>,
    pub locations: Vec<Location>,
    pub link_names: Vec<LinkName>,
}

pub struct Location {
    pub name: Name,
    pub note: Note,
    pub links: Vec<Link>,
    pub objects: Vec<ObjectId>,
    pub entities: Vec<EntityId>,
}

pub struct Link {
    pub link_name: LinkNameId,
    pub location: LocationId,
}

pub struct LinkName {
    pub name: Name,
}

pub struct Object {
    pub name: Name,
}

pub struct Entity {
    pub name: Name,
    pub location: LocationId,
    pub objects: Vec<ObjectId>,
    pub messages: Vec<Message>,
}
