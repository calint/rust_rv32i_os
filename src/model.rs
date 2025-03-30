use alloc::vec;
use alloc::vec::Vec;
use core::ops::Deref;

pub type LocationId = usize;
pub type LinkId = usize;
pub type EntityId = usize;
pub type ObjectId = usize;
pub type Name = FixedSizeCStr<32>;
pub type Note = FixedSizeCStr<64>;
pub type EntityMessage = FixedSizeCStr<128>;

pub struct World {
    pub objects: Vec<Object>,
    pub entities: Vec<Entity>,
    pub locations: Vec<Location>,
    pub links: Vec<Link>,
}

impl World {
    pub fn find_or_add_link(&mut self, link_name: &[u8]) -> LinkId {
        match self.links.iter().position(|x| x.name == link_name) {
            Some(id) => id,
            None => {
                let id = self.links.len();
                self.links.push(Link {
                    name: FixedSizeCStr::from(link_name),
                });
                id
            }
        }
    }

    pub fn add_object(&mut self, object_name: &[u8]) -> ObjectId {
        let object_id = self.objects.len();
        self.objects.push(Object {
            name: FixedSizeCStr::from(object_name),
        });
        object_id
    }

    pub fn add_entity(&mut self, entity_name: &[u8], location_id: LocationId) -> EntityId {
        let entity_id = self.entities.len();
        self.entities.push(Entity {
            name: FixedSizeCStr::from(entity_name),
            location: location_id,
            objects: vec![],
            messages: vec![],
        });
        self.locations[location_id].entities.push(entity_id);
        entity_id
    }
}

pub struct Location {
    pub name: Name,
    pub note: Note,
    pub links: Vec<LocationLink>,
    pub objects: Vec<ObjectId>,
    pub entities: Vec<EntityId>,
}

#[derive(Clone, Copy)]
pub struct FixedSizeCStr<const N: usize> {
    data: [u8; N],
    len: usize,
}

impl<const N: usize> FixedSizeCStr<N> {
    pub fn new() -> Self {
        Self {
            data: [0u8; N],
            len: 0,
        }
    }

    pub fn from(src: &[u8]) -> Self {
        FixedSizeCStr::from_parts(&[src])
    }

    pub fn from_parts(parts: &[&[u8]]) -> Self {
        let mut s = Self::new();
        for &part in parts {
            s.append(part);
        }
        s
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn append(&mut self, s: &[u8]) -> &Self {
        let cpy_len = s.len().min(N - self.len);
        self.data[self.len..self.len + cpy_len].copy_from_slice(&s[..cpy_len]);
        self.len += cpy_len;
        self
    }
}

impl<const N: usize> Default for FixedSizeCStr<N> {
    fn default() -> Self {
        Self {
            data: [0u8; N],
            len: 0,
        }
    }
}

impl<const N: usize> Deref for FixedSizeCStr<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data[..self.len]
    }
}

impl<const N: usize> PartialEq<&[u8]> for FixedSizeCStr<N> {
    fn eq(&self, other: &&[u8]) -> bool {
        self.deref() == *other
    }
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

pub fn find_object_in_entity_inventory(
    world: &World,
    entity_id: EntityId,
    object_name: &[u8],
) -> Option<(usize, ObjectId)> {
    world.entities[entity_id]
        .objects
        .iter()
        .enumerate()
        .find_map(|(index, &oid)| {
            if world.objects[oid].name == object_name {
                Some((index, oid))
            } else {
                None
            }
        })
}

pub fn send_message_to_location_entities(
    world: &mut World,
    location_id: LocationId,
    exclude_entities_id: &[EntityId],
    message: EntityMessage,
) {
    for &eid in &world.locations[location_id].entities {
        if !exclude_entities_id.contains(&eid) {
            world.entities[eid].messages.push(message);
        }
    }
}

pub fn send_message_to_entities(world: &mut World, entities: &[EntityId], message: EntityMessage) {
    for &eid in entities {
        world.entities[eid].messages.push(message);
    }
}
