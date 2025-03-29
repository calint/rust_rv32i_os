use alloc::vec;
use alloc::vec::Vec;

const NAME_SIZE: usize = 32;
const NOTE_SIZE: usize = 64;
const ENTITY_MESSAGE_SIZE: usize = 64;

pub type LocationId = usize;
pub type LinkId = usize;
pub type EntityId = usize;
pub type ObjectId = usize;

pub struct World {
    pub objects: Vec<Object>,
    pub entities: Vec<Entity>,
    pub locations: Vec<Location>,
    pub links: Vec<Link>,
}

impl World {
    pub fn find_or_add_link(&mut self, link_name: &[u8]) -> LinkId {
        match self.links.iter().position(|x| x.name.equals(link_name)) {
            Some(id) => id,
            None => {
                let id = self.links.len();
                self.links.push(Link {
                    name: Name::from(link_name),
                });
                id
            }
        }
    }

    pub fn add_object(&mut self, object_name: &[u8]) -> ObjectId {
        let object_id = self.objects.len();
        self.objects.push(Object {
            name: Name::from(object_name),
        });
        object_id
    }

    pub fn add_entity(&mut self, entity_name: &[u8], location_id: LocationId) -> EntityId {
        let entity_id = self.entities.len();
        self.entities.push(Entity {
            name: Name::from(entity_name),
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

pub struct Name {
    pub data: [u8; NAME_SIZE],
}

impl Name {
    pub fn new() -> Self {
        Self {
            data: [0u8; NAME_SIZE],
        }
    }

    pub fn from(src: &[u8]) -> Self {
        let mut name = Self::new();
        let len = src.len().min(NAME_SIZE - 1);
        // note: -1 to enabled string terminator at the end of string
        name.data[..len].copy_from_slice(&src[..len]);
        name
    }

    pub fn equals(&self, compare_with: &[u8]) -> bool {
        if compare_with.len() >= NAME_SIZE {
            // note: >= to ensure the end of string terminator can be compared
            return false;
        }
        self.data.starts_with(compare_with) && self.data[compare_with.len()] == 0
    }
}

pub struct Note {
    pub data: [u8; NOTE_SIZE],
}

impl Note {
    pub fn new() -> Self {
        Self {
            data: [0u8; NOTE_SIZE],
        }
    }

    pub fn from(src: &[u8]) -> Self {
        let mut note = Self::new();
        let len = src.len().min(NOTE_SIZE - 1);
        // note: -1 to enabled string terminator at the end of string
        note.data[..len].copy_from_slice(&src[..len]);
        note
    }

    pub fn is_empty(&self) -> bool {
        self.data[0] == 0
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

#[derive(Clone)]
pub struct EntityMessage {
    pub data: [u8; ENTITY_MESSAGE_SIZE],
}

impl EntityMessage {
    pub fn new() -> Self {
        Self {
            data: [0u8; ENTITY_MESSAGE_SIZE],
        }
    }
    pub fn from(parts: &[&[u8]]) -> Self {
        let mut message = EntityMessage::new();
        set_u8_buffer_from_parts(&mut message.data, parts);
        message
    }
}

fn set_u8_buffer_from_parts(buffer: &mut [u8], parts: &[&[u8]]) {
    let mut index = 0;

    // Helper to copy a part into the buffer, considering null termination
    fn copy_part(buffer: &mut [u8], index: &mut usize, part: &[u8]) {
        let part_len = part.iter().position(|&c| c == 0).unwrap_or(part.len());
        for &byte in &part[..part_len] {
            if *index >= buffer.len() {
                break;
            }
            buffer[*index] = byte;
            *index += 1;
        }
    }

    // Copy each part into the buffer
    for &part in parts {
        copy_part(buffer, &mut index, part);
    }

    // Null-terminate the buffer if there's space
    if index < buffer.len() {
        buffer[index] = 0;
    }
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
            if world.objects[oid].name.equals(object_name) {
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
            world.entities[eid].messages.push(message.clone());
        }
    }
}

pub fn send_message_to_entities(world: &mut World, entities: &[EntityId], message: EntityMessage) {
    for &eid in entities {
        world.entities[eid].messages.push(message.clone());
    }
}
