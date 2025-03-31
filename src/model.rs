use super::lib::fixed_size_string::FixedSizeString;
use alloc::vec;
use alloc::vec::Vec;

pub type LocationId = usize;
pub type LinkId = usize;
pub type EntityId = usize;
pub type ObjectId = usize;
pub type Name = FixedSizeString<32>;
pub type Note = FixedSizeString<64>;
pub type EntityMessage = FixedSizeString<128>;

pub struct World {
    pub objects: Vec<Object>,
    pub entities: Vec<Entity>,
    pub locations: Vec<Location>,
    pub links: Vec<Link>,
}

impl World {
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

    pub fn find_object_in_entity_inventory(
        &self,
        entity_id: EntityId,
        object_name: &[u8],
    ) -> Option<(usize, ObjectId)> {
        self.entities[entity_id]
            .objects
            .iter()
            .enumerate()
            .find_map(|(index, &oid)| {
                if self.objects[oid].name == object_name {
                    Some((index, oid))
                } else {
                    None
                }
            })
    }

    pub fn find_or_add_link(&mut self, link_name: &[u8]) -> LinkId {
        if let Some(id) = self.links.iter().position(|x| x.name == link_name) { id } else {
            let id = self.links.len();
            self.links.push(Link {
                name: Name::from(link_name),
            });
            id
        }
    }

    pub fn send_message_to_location_entities(
        &mut self,
        location_id: LocationId,
        exclude_entities: &[EntityId],
        message: EntityMessage,
    ) {
        for &eid in &self.locations[location_id].entities {
            if !exclude_entities.contains(&eid) {
                self.entities[eid].messages.push(message);
            }
        }
    }

    pub fn send_message_to_entities(&mut self, entities: &[EntityId], message: EntityMessage) {
        for &eid in entities {
            self.entities[eid].messages.push(message);
        }
    }
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
