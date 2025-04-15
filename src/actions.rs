use crate::lib::api::{
    memory_end, memory_heap_start, u8_slice_to_u32, uart_send_bytes, uart_send_hex_u32,
};
use crate::lib::api_unsafe::{
    SDCARD_SECTOR_SIZE_BYTES, led_set, memory_stack_pointer, sdcard_read_blocking, sdcard_status,
    sdcard_write_blocking, uart_send_byte,
};
use crate::lib::global_allocator::GlobalAllocator;
use crate::model::{EntityId, World};
use crate::model::{EntityMessage, Location, LocationLink, Name, Note};
use crate::{CommandBufferIterator, HELP};
use alloc::vec;

pub type Result<T> = core::result::Result<T, ActionFailed>;

pub enum ActionFailed {
    InvalidCommand,
    GoWhere,
    NoSuchExit,
    CannotGoThere,
    TakeWhat,
    ObjectNotHere,
    DropWhat,
    ObjectNotInInventory,
    GiveToWhom,
    GiveWhat,
    EntityNotHere,
    WhatSector,
    WhichLeds,
    WhatObjectName,
    ObjectAlreadyExists,
    WhatToLinkName,
    WhatBackLinkName,
    WhatNewLocationName,
    LocationAlreadyExists,
    LinkFromLocationAlreadyExists,
    WhatEntityName,
    EntityAlreadyExists,
    SayWhat,
    TellToWhom,
    TellWhat,
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "actions return Result for consistency"
)]
pub fn action_look(world: &mut World, entity_id: EntityId) -> Result<()> {
    let entity = &mut world.entities[entity_id];
    let location = &world.locations[entity.location];

    let messages = &entity.messages;
    for x in messages {
        uart_send_bytes(x);
        uart_send_bytes(b"\r\n");
    }

    // clear messages after displayed
    entity.messages.clear();

    uart_send_bytes(b"u r in ");
    uart_send_bytes(&location.name);

    uart_send_bytes(b"\r\nu c ");
    let mut i = 0;
    for &eid in &location.entities {
        if eid != entity_id {
            if i != 0 {
                uart_send_bytes(b", ");
            }
            uart_send_bytes(&world.entities[eid].name);
            i += 1;
        }
    }
    for &oid in &location.objects {
        if i != 0 {
            uart_send_bytes(b", ");
        }
        i += 1;
        uart_send_bytes(&world.objects[oid].name);
    }
    if i == 0 {
        uart_send_bytes(b"nothing");
    }
    uart_send_bytes(b"\r\n");

    uart_send_bytes(b"exits: ");
    i = 0;
    for lid in &location.links {
        if i != 0 {
            uart_send_bytes(b", ");
        }
        i += 1;
        uart_send_bytes(&world.links[lid.link].name);
    }
    if i == 0 {
        uart_send_bytes(b"none");
    }
    uart_send_bytes(b"\r\n");

    if !location.note.is_empty() {
        uart_send_bytes(&location.note);
        uart_send_bytes(b"\r\n");
    }

    Ok(())
}

pub fn action_go(
    world: &mut World,
    entity_id: EntityId,
    it: &mut CommandBufferIterator,
) -> Result<()> {
    let Some(named_link) = it.next() else {
        uart_send_bytes(b"go where\r\n\r\n");
        return Err(ActionFailed::GoWhere);
    };

    action_go_named_link(world, entity_id, named_link)
}

pub fn action_go_named_link(
    world: &mut World,
    entity_id: EntityId,
    link_name: &[u8],
) -> Result<()> {
    // find link id
    let Some(link_id) = world.links.iter().position(|x| x.name == link_name) else {
        uart_send_bytes(b"no such exit\r\n\r\n");
        return Err(ActionFailed::NoSuchExit);
    };

    // move entity
    let (from_location_id, to_location_id) = {
        let entity = &mut world.entities[entity_id];
        let from_location_id = entity.location;
        let from_location = &mut world.locations[from_location_id];

        // find "to" location id
        let to_location_id =
            if let Some(lnk) = from_location.links.iter().find(|x| x.link == link_id) {
                lnk.location
            } else {
                uart_send_bytes(b"cannot go there\r\n\r\n");
                return Err(ActionFailed::CannotGoThere);
            };

        // remove entity from old location
        let pos = from_location
            .entities
            .iter()
            .position(|&x| x == entity_id)
            .expect("entity should be in location");

        from_location.entities.remove(pos);

        // add entity to new location
        world.locations[to_location_id].entities.push(entity_id);

        // update entity location
        entity.location = to_location_id;

        (from_location_id, to_location_id)
    };

    // send message to entities in 'from_location' that entity has left
    world.send_message_to_entities_in_location(
        from_location_id,
        &[entity_id],
        EntityMessage::from_parts(&[&world.entities[entity_id].name, b" left to ", link_name]),
    );

    // find link name that leads from 'to_location_id' to 'from_location_id'
    // note: assumes links are bi-directional thus panic if not
    let back_link_id = world.locations[to_location_id]
        .links
        .iter()
        .find_map(|x| (x.location == from_location_id).then_some(x.link))
        .expect("link back to location should exist in target location");

    // send message to entities in 'to_location' that entity has arrived
    world.send_message_to_entities_in_location(
        to_location_id,
        &[entity_id],
        EntityMessage::from_parts(&[
            &world.entities[entity_id].name,
            b" arrived from ",
            &world.links[back_link_id].name,
        ]),
    );

    Ok(())
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "actions return Result for consistency"
)]
pub fn action_inventory(world: &World, entity_id: EntityId) -> Result<()> {
    let entity = &world.entities[entity_id];
    uart_send_bytes(b"u have: ");
    let mut i = 0;
    for &oid in &entity.objects {
        if i != 0 {
            uart_send_bytes(b", ");
        }
        i += 1;
        uart_send_bytes(&world.objects[oid].name);
    }
    if i == 0 {
        uart_send_bytes(b"nothing");
    }
    uart_send_bytes(b"\r\n");

    Ok(())
}

pub fn action_take(
    world: &mut World,
    entity_id: EntityId,
    it: &mut CommandBufferIterator,
) -> Result<()> {
    // get object name
    let Some(object_name) = it.next() else {
        uart_send_bytes(b"take what\r\n\r\n");
        return Err(ActionFailed::TakeWhat);
    };

    {
        let entity = &mut world.entities[entity_id];
        let location = &mut world.locations[entity.location];

        // find object id and index in list
        let Some((object_index, &object_id)) = location
            .objects
            .iter()
            .enumerate()
            .find(|&(_, &oid)| world.objects[oid].name == object_name)
        else {
            uart_send_bytes(object_name);
            uart_send_bytes(b" is not here\r\n\r\n");
            return Err(ActionFailed::ObjectNotHere);
        };

        // remove object from location
        location.objects.remove(object_index);

        // add object to entity
        entity.objects.push(object_id);
    }

    // send message
    {
        let entity = &world.entities[entity_id];
        world.send_message_to_entities_in_location(
            entity.location,
            &[entity_id],
            EntityMessage::from_parts(&[&entity.name, b" took ", object_name]),
        );
    }

    Ok(())
}

pub fn action_drop(
    world: &mut World,
    entity_id: EntityId,
    it: &mut CommandBufferIterator,
) -> Result<()> {
    let Some(object_name) = it.next() else {
        uart_send_bytes(b"drop what\r\n\r\n");
        return Err(ActionFailed::DropWhat);
    };

    {
        let Some((object_index, object_id)) =
            world.find_object_in_entity_inventory(entity_id, object_name)
        else {
            uart_send_bytes(object_name);
            uart_send_bytes(b" not in inventory\r\n\r\n");
            return Err(ActionFailed::ObjectNotInInventory);
        };

        let entity = &mut world.entities[entity_id];

        // remove object from entity
        entity.objects.remove(object_index);

        // add object to location
        world.locations[entity.location].objects.push(object_id);
    }

    // send message
    {
        let entity = &world.entities[entity_id];
        world.send_message_to_entities_in_location(
            entity.location,
            &[entity_id],
            EntityMessage::from_parts(&[&entity.name, b" dropped ", object_name]),
        );
    }

    Ok(())
}

pub fn action_give(
    world: &mut World,
    entity_id: EntityId,
    it: &mut CommandBufferIterator,
) -> Result<()> {
    // get entity name
    let Some(to_entity_name) = it.next() else {
        uart_send_bytes(b"give to whom\r\n\r\n");
        return Err(ActionFailed::GiveToWhom);
    };

    // get object name
    let Some(object_name) = it.next() else {
        uart_send_bytes(b"give what\r\n\r\n");
        return Err(ActionFailed::GiveWhat);
    };

    let Some((object_index, object_id)) =
        world.find_object_in_entity_inventory(entity_id, object_name)
    else {
        uart_send_bytes(object_name);
        uart_send_bytes(b" not in inventory\r\n\r\n");
        return Err(ActionFailed::ObjectNotInInventory);
    };

    // find "to" entity
    let Some(&to_entity_id) = world.locations[world.entities[entity_id].location]
        .entities
        .iter()
        .find(|&&x| world.entities[x].name == to_entity_name)
    else {
        uart_send_bytes(to_entity_name);
        uart_send_bytes(b" not here\r\n\r\n");
        return Err(ActionFailed::EntityNotHere);
    };

    // remove object from entity
    world.entities[entity_id].objects.remove(object_index);

    // add object to "to" entity
    world.entities[to_entity_id].objects.push(object_id);

    // send messages
    world.send_message_to_entities_in_location(
        world.entities[entity_id].location,
        &[to_entity_id],
        EntityMessage::from_parts(&[
            &world.entities[entity_id].name,
            b" gave ",
            &world.entities[to_entity_id].name,
            b" ",
            &world.objects[object_id].name,
        ]),
    );

    world.send_message_to_entities(
        &[to_entity_id],
        EntityMessage::from_parts(&[
            &world.entities[entity_id].name,
            b" gave u ",
            &world.objects[object_id].name,
        ]),
    );

    Ok(())
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "actions return Result for consistency"
)]
pub fn action_memory_info() -> Result<()> {
    uart_send_bytes(b"   heap start: ");
    uart_send_hex_u32(memory_heap_start(), true);
    uart_send_bytes(b"\r\nstack pointer: ");
    uart_send_hex_u32(memory_stack_pointer(), true);
    uart_send_bytes(b"\r\n   memory end: ");
    uart_send_hex_u32(memory_end(), true);
    uart_send_bytes(b"\r\n\r\nheap blocks:\r\n");
    GlobalAllocator::debug_block_list();
    uart_send_bytes(b"\r\n");

    Ok(())
}

#[expect(clippy::cast_sign_loss, reason = "intended behavior")]
#[allow(
    clippy::unnecessary_wraps,
    reason = "actions return Result for consistency"
)]
pub fn action_sdcard_status() -> Result<()> {
    uart_send_bytes(b"SDCARD_STATUS: 0x");
    uart_send_hex_u32(sdcard_status() as u32, true);
    uart_send_bytes(b"\r\n");

    Ok(())
}

pub fn action_sdcard_read(it: &mut CommandBufferIterator) -> Result<()> {
    let sector = if let Some(sector) = it.next() {
        u8_slice_to_u32(sector)
    } else {
        uart_send_bytes(b"what sector\r\n");
        return Err(ActionFailed::WhatSector);
    };

    let mut buf = [0_u8; SDCARD_SECTOR_SIZE_BYTES];
    sdcard_read_blocking(sector, &mut buf);
    buf.iter().for_each(|&x| uart_send_byte(x));
    uart_send_bytes(b"\r\n");

    Ok(())
}

pub fn action_sdcard_write(it: &mut CommandBufferIterator) -> Result<()> {
    let sector = if let Some(sector) = it.next() {
        u8_slice_to_u32(sector)
    } else {
        uart_send_bytes(b"what sector\r\n");
        return Err(ActionFailed::WhatSector);
    };

    let data = it.rest();
    let len = data.len().min(SDCARD_SECTOR_SIZE_BYTES);
    let mut buf = [0_u8; SDCARD_SECTOR_SIZE_BYTES];
    buf[..len].copy_from_slice(&data[..len]);
    sdcard_write_blocking(sector, &buf);

    Ok(())
}

#[expect(clippy::cast_possible_truncation, reason = "intended behavior")]
pub fn action_led_set(it: &mut CommandBufferIterator) -> Result<()> {
    let bits = if let Some(bits) = it.next() {
        u8_slice_to_u32(bits)
    } else {
        uart_send_bytes(b"which leds (in bits as decimal with 0 being on)\r\n");
        return Err(ActionFailed::WhichLeds);
    };

    led_set(bits as u8);

    Ok(())
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "actions return Result for consistency"
)]
pub fn action_help() -> Result<()> {
    uart_send_bytes(HELP);

    Ok(())
}

pub fn action_new_object(
    world: &mut World,
    entity_id: EntityId,
    it: &mut CommandBufferIterator,
) -> Result<()> {
    // get object name
    let Some(object_name) = it.next() else {
        uart_send_bytes(b"what object name\r\n");
        return Err(ActionFailed::WhatObjectName);
    };

    if world.objects.iter().any(|x| x.name == object_name) {
        uart_send_bytes(b"object already exists\r\n");
        return Err(ActionFailed::ObjectAlreadyExists);
    }

    let object_id = world.add_object(object_name);

    world.entities[entity_id].objects.push(object_id);

    Ok(())
}

pub fn action_new_location(
    world: &mut World,
    entity_id: EntityId,
    it: &mut CommandBufferIterator,
) -> Result<()> {
    let Some(to_link_name) = it.next() else {
        uart_send_bytes(b"what link name\r\n");
        return Err(ActionFailed::WhatToLinkName);
    };

    let Some(back_link_name) = it.next() else {
        uart_send_bytes(b"what back link name\r\n");
        return Err(ActionFailed::WhatBackLinkName);
    };

    let Some(new_location_name) = it.next() else {
        uart_send_bytes(b"what new location name\r\n");
        return Err(ActionFailed::WhatNewLocationName);
    };

    if world.locations.iter().any(|x| x.name == new_location_name) {
        uart_send_bytes(b"location already exists\r\n");
        return Err(ActionFailed::LocationAlreadyExists);
    }

    let from_location_id = world.entities[entity_id].location;

    let to_link_id = world.find_or_add_link(to_link_name);

    // check if link is already used
    if world.locations[from_location_id]
        .links
        .iter()
        .any(|x| x.link == to_link_id)
    {
        uart_send_bytes(b"link from this location already exists\r\n");
        return Err(ActionFailed::LinkFromLocationAlreadyExists);
    }

    let back_link_id = world.find_or_add_link(back_link_name);

    // add location and link it back to from location
    let new_location_id = world.locations.len();
    world.locations.push(Location {
        name: Name::from(new_location_name),
        note: Note::default(),
        links: vec![LocationLink {
            link: back_link_id,
            location: from_location_id,
        }],
        objects: vec![],
        entities: vec![],
    });

    world.locations[from_location_id].links.push(LocationLink {
        link: to_link_id,
        location: new_location_id,
    });

    Ok(())
}

pub fn action_new_entity(
    world: &mut World,
    entity_id: EntityId,
    it: &mut CommandBufferIterator,
) -> Result<()> {
    // get object name
    let Some(entity_name) = it.next() else {
        uart_send_bytes(b"what entity name\r\n");
        return Err(ActionFailed::WhatEntityName);
    };

    if world.entities.iter().any(|x| x.name == entity_name) {
        uart_send_bytes(b"entity already exists\r\n");
        return Err(ActionFailed::EntityAlreadyExists);
    }

    world.add_entity(entity_name, world.entities[entity_id].location);

    Ok(())
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "actions return Result for consistency"
)]
pub fn action_set_location_note(
    world: &mut World,
    entity_id: EntityId,
    it: &mut CommandBufferIterator,
) -> Result<()> {
    world.locations[world.entities[entity_id].location].note = Note::from(it.rest());

    Ok(())
}

pub fn action_say(
    world: &mut World,
    entity_id: EntityId,
    it: &mut CommandBufferIterator,
) -> Result<()> {
    let say = it.rest();
    if say.is_empty() {
        uart_send_bytes(b"say what");
        return Err(ActionFailed::SayWhat);
    }

    let entity = &world.entities[entity_id];
    world.send_message_to_entities_in_location(
        entity.location,
        &[entity_id],
        EntityMessage::from_parts(&[&entity.name, b" says ", say]),
    );

    Ok(())
}

pub fn action_tell(
    world: &mut World,
    entity_id: EntityId,
    it: &mut CommandBufferIterator,
) -> Result<()> {
    let Some(to_name) = it.next() else {
        uart_send_bytes(b"tell to whom\r\n");
        return Err(ActionFailed::TellToWhom);
    };

    let tell = it.rest();
    if tell.is_empty() {
        uart_send_bytes(b"tell what\r\n");
        return Err(ActionFailed::TellWhat);
    }

    let entity = &world.entities[entity_id];

    let Some(&to_entity_id) = world.locations[entity.location]
        .entities
        .iter()
        .find(|&&x| world.entities[x].name == to_name)
    else {
        uart_send_bytes(to_name);
        uart_send_bytes(b" not here\r\n");
        return Err(ActionFailed::EntityNotHere);
    };

    let message = EntityMessage::from_parts(&[&entity.name, b" tells u ", tell]);
    world.entities[to_entity_id].messages.push(message);

    Ok(())
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "actions return Result for consistency"
)]
pub const fn action_wait() -> Result<()> {
    Ok(())
}
