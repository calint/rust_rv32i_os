//
// reviewed: 2025-04-21
//
use crate::lib::api::{Leds, Memory, Printer, SDCard, u8_slice_bits_to_u32, u8_slice_to_u32};
use crate::lib::cursor_buffer::{CursorBuffer, CursorBufferIterator};
use crate::lib::global_allocator::GlobalAllocator;
use crate::model::{
    Entity, EntityId, Link, LinkName, LinkNameId, Location, LocationId, Message, Name, Note,
    Object, ObjectId, World,
};
use alloc::vec;

const COMMAND_BUFFER_SIZE: usize = 526;
// note: enough to support write to SD card sector of 512 byte in 4 GB address space

pub type CommandBuffer = CursorBuffer<COMMAND_BUFFER_SIZE, u8>;
pub type CommandBufferIterator<'a> =
    CursorBufferIterator<'a, COMMAND_BUFFER_SIZE, u8, fn(&u8) -> bool>;

pub type Result<T> = core::result::Result<T, Error>;

pub enum Error {
    NotUnderstood,
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

pub struct ActionContext<'a> {
    pub printer: &'a mut dyn Printer,
    pub world: &'a mut World,
    pub entity: EntityId,
    pub tokens: &'a mut CommandBufferIterator<'a>,
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "actions return Result for consistency"
)]
pub fn look(ctx: &mut ActionContext) -> Result<()> {
    let entity = &ctx.world.entities[ctx.entity];
    let location = &ctx.world.locations[entity.location];

    ctx.printer.p(b"u r in ");
    ctx.printer.p(&location.name);
    ctx.printer.nl();

    ctx.printer.p(b"u c ");
    let mut count = 0;
    for &eid in &location.entities {
        if eid != ctx.entity {
            if count != 0 {
                ctx.printer.p(b", ");
            }
            ctx.printer.p(&ctx.world.entities[eid].name);
            count += 1;
        }
    }
    for &oid in &location.objects {
        if count != 0 {
            ctx.printer.p(b", ");
        }
        count += 1;
        ctx.printer.p(&ctx.world.objects[oid].name);
    }
    if count == 0 {
        ctx.printer.p(b"nothing");
    }
    ctx.printer.nl();

    ctx.printer.p(b"exits: ");
    count = 0;
    for lid in &location.links {
        if count != 0 {
            ctx.printer.p(b", ");
        }
        count += 1;
        ctx.printer.p(&ctx.world.link_names[lid.link_name].name);
    }
    if count == 0 {
        ctx.printer.p(b"none");
    }
    ctx.printer.nl();

    if !location.note.is_empty() {
        ctx.printer.pl(&location.note);
    }

    for x in &entity.messages {
        ctx.printer.pl(x);
    }

    // clear messages after displayed
    ctx.world.entities[ctx.entity].messages.clear();

    Ok(())
}

pub fn go(ctx: &mut ActionContext) -> Result<()> {
    let Some(named_link) = ctx.tokens.next() else {
        ctx.printer.p(b"go where");
        ctx.printer.nlc(2);
        return Err(Error::GoWhere);
    };

    go_named_link(ctx, named_link)
}

pub fn go_named_link(ctx: &mut ActionContext, link_name: &[u8]) -> Result<()> {
    // find link id
    let Some(link_name_id) = ctx
        .world
        .link_names
        .iter()
        .position(|x| x.name == link_name)
    else {
        ctx.printer.p(b"cannot go there");
        ctx.printer.nlc(2);
        return Err(Error::NoSuchExit);
    };

    // move entity
    let (from_location_id, to_location_id) = {
        let entity = &mut ctx.world.entities[ctx.entity];
        let from_location_id = entity.location;
        let from_location = &mut ctx.world.locations[from_location_id];

        // find "to" location id
        let to_location_id = if let Some(lnk) = from_location
            .links
            .iter()
            .find(|x| x.link_name == link_name_id)
        {
            lnk.location
        } else {
            ctx.printer.p(b"cannot go there");
            ctx.printer.nlc(2);
            return Err(Error::CannotGoThere);
        };

        // remove entity from old location
        let pos = from_location
            .entities
            .iter()
            .position(|&x| x == ctx.entity)
            .expect("entity should be in location");

        from_location.entities.remove(pos);

        // add entity to new location
        ctx.world.locations[to_location_id]
            .entities
            .push(ctx.entity);

        // update entity location
        entity.location = to_location_id;

        (from_location_id, to_location_id)
    };

    // send message to entities in 'from_location' that entity has left
    send_message_to_entities_in_location(
        ctx.world,
        from_location_id,
        &[ctx.entity],
        Message::from_parts(&[
            &ctx.world.entities[ctx.entity].name,
            b" left to ",
            link_name,
        ]),
    );

    // find link name that leads from 'to_location_id' to 'from_location_id'
    // note: assumes links are bi-directional thus panic if not
    let back_link_id = ctx.world.locations[to_location_id]
        .links
        .iter()
        .find_map(|x| (x.location == from_location_id).then_some(x.link_name))
        .expect("link back to location should exist in target location");

    // send message to entities in 'to_location' that entity has arrived
    send_message_to_entities_in_location(
        ctx.world,
        to_location_id,
        &[ctx.entity],
        Message::from_parts(&[
            &ctx.world.entities[ctx.entity].name,
            b" arrived from ",
            &ctx.world.link_names[back_link_id].name,
        ]),
    );

    Ok(())
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "actions return Result for consistency"
)]
pub fn inventory(ctx: &mut ActionContext) -> Result<()> {
    ctx.printer.p(b"u have: ");
    let mut i = 0;
    for &oid in &ctx.world.entities[ctx.entity].objects {
        if i != 0 {
            ctx.printer.p(b", ");
        }
        i += 1;
        ctx.printer.p(&ctx.world.objects[oid].name);
    }
    if i == 0 {
        ctx.printer.p(b"nothing");
    }
    ctx.printer.nl();

    Ok(())
}

pub fn take(ctx: &mut ActionContext) -> Result<()> {
    // get object name
    let Some(object_name) = ctx.tokens.next() else {
        ctx.printer.p(b"take what");
        ctx.printer.nlc(2);
        return Err(Error::TakeWhat);
    };

    {
        let entity = &mut ctx.world.entities[ctx.entity];
        let location = &mut ctx.world.locations[entity.location];

        // find object id and index in list
        let Some((object_index, &object_id)) = location
            .objects
            .iter()
            .enumerate()
            .find(|&(_, &oid)| ctx.world.objects[oid].name == object_name)
        else {
            ctx.printer.p(object_name);
            ctx.printer.p(b" not here");
            ctx.printer.nlc(2);
            return Err(Error::ObjectNotHere);
        };

        // remove object from location
        location.objects.remove(object_index);

        // add object to entity
        entity.objects.push(object_id);
    }

    // send message
    {
        let entity = &ctx.world.entities[ctx.entity];
        send_message_to_entities_in_location(
            ctx.world,
            entity.location,
            &[ctx.entity],
            Message::from_parts(&[&entity.name, b" took ", object_name]),
        );
    }

    Ok(())
}

pub fn drop(ctx: &mut ActionContext) -> Result<()> {
    let Some(object_name) = ctx.tokens.next() else {
        ctx.printer.p(b"drop what");
        ctx.printer.nlc(2);
        return Err(Error::DropWhat);
    };

    {
        let Some((object_index, object_id)) =
            find_object_in_entity_inventory(ctx.world, ctx.entity, object_name)
        else {
            ctx.printer.p(object_name);
            ctx.printer.p(b" not in inventory");
            ctx.printer.nlc(2);
            return Err(Error::ObjectNotInInventory);
        };

        let entity = &mut ctx.world.entities[ctx.entity];

        // remove object from entity
        entity.objects.remove(object_index);

        // add object to location
        ctx.world.locations[entity.location].objects.push(object_id);
    }

    // send message
    {
        let entity = &ctx.world.entities[ctx.entity];
        send_message_to_entities_in_location(
            ctx.world,
            entity.location,
            &[ctx.entity],
            Message::from_parts(&[&entity.name, b" dropped ", object_name]),
        );
    }

    Ok(())
}

pub fn give(ctx: &mut ActionContext) -> Result<()> {
    // get entity name
    let Some(to_entity_name) = ctx.tokens.next() else {
        ctx.printer.p(b"give to whom");
        ctx.printer.nlc(2);
        return Err(Error::GiveToWhom);
    };

    // get object name
    let Some(object_name) = ctx.tokens.next() else {
        ctx.printer.p(b"give what");
        ctx.printer.nlc(2);
        return Err(Error::GiveWhat);
    };

    let Some((object_index, object_id)) =
        find_object_in_entity_inventory(ctx.world, ctx.entity, object_name)
    else {
        ctx.printer.p(object_name);
        ctx.printer.p(b" not in inventory");
        ctx.printer.nlc(2);
        return Err(Error::ObjectNotInInventory);
    };

    // find "to" entity
    let Some(&to_entity_id) = ctx.world.locations[ctx.world.entities[ctx.entity].location]
        .entities
        .iter()
        .find(|&&x| ctx.world.entities[x].name == to_entity_name)
    else {
        ctx.printer.p(to_entity_name);
        ctx.printer.p(b" not here");
        ctx.printer.nlc(2);
        return Err(Error::EntityNotHere);
    };

    // remove object from entity
    ctx.world.entities[ctx.entity].objects.remove(object_index);

    // add object to "to" entity
    ctx.world.entities[to_entity_id].objects.push(object_id);

    // send messages
    send_message_to_entities_in_location(
        ctx.world,
        ctx.world.entities[ctx.entity].location,
        &[to_entity_id],
        Message::from_parts(&[
            &ctx.world.entities[ctx.entity].name,
            b" gave ",
            &ctx.world.entities[to_entity_id].name,
            b" ",
            &ctx.world.objects[object_id].name,
        ]),
    );

    send_message_to_entities(
        ctx.world,
        &[to_entity_id],
        Message::from_parts(&[
            &ctx.world.entities[ctx.entity].name,
            b" gave u ",
            &ctx.world.objects[object_id].name,
        ]),
    );

    Ok(())
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "actions return Result for consistency"
)]
pub fn memory_info(ctx: &mut ActionContext) -> Result<()> {
    ctx.printer.p(b"   heap start: ");
    ctx.printer.p_hex_u32(Memory::heap_start(), true);
    ctx.printer.nl();
    ctx.printer.p(b"stack pointer: ");
    ctx.printer.p_hex_u32(Memory::stack_pointer(), true);
    ctx.printer.nl();
    ctx.printer.p(b"   memory end: ");
    ctx.printer.p_hex_u32(Memory::end(), true);
    ctx.printer.nl();
    ctx.printer.nl();
    ctx.printer.p(b"heap blocks:");
    ctx.printer.nl();
    GlobalAllocator::debug_block_list(ctx.printer);

    Ok(())
}

#[expect(clippy::cast_sign_loss, reason = "intended behavior")]
#[allow(
    clippy::unnecessary_wraps,
    reason = "actions return Result for consistency"
)]
pub fn sdcard_status(ctx: &mut ActionContext) -> Result<()> {
    ctx.printer.p(b"SDCARD_STATUS: 0x");
    ctx.printer.p_hex_u32(SDCard::status() as u32, true);
    ctx.printer.nl();

    Ok(())
}

pub fn sdcard_read(ctx: &mut ActionContext) -> Result<()> {
    let sector = if let Some(sector) = ctx.tokens.next() {
        u8_slice_to_u32(sector)
    } else {
        ctx.printer.p(b"what sector");
        ctx.printer.nlc(2);
        return Err(Error::WhatSector);
    };

    let mut buf = [0_u8; SDCard::sector_size_bytes()];
    SDCard::read_blocking(sector, &mut buf);
    buf.iter().for_each(|&x| ctx.printer.pb(x));
    ctx.printer.nl();

    Ok(())
}

pub fn sdcard_write(ctx: &mut ActionContext) -> Result<()> {
    let sector = if let Some(sector) = ctx.tokens.next() {
        u8_slice_to_u32(sector)
    } else {
        ctx.printer.p(b"what sector");
        ctx.printer.nlc(2);
        return Err(Error::WhatSector);
    };

    let data = ctx.tokens.rest();
    let len = data.len().min(SDCard::sector_size_bytes());
    let mut buf = [0_u8; SDCard::sector_size_bytes()];
    buf[..len].copy_from_slice(&data[..len]);
    // todo: allow slice to be less than sector size on pad rest with zeros
    SDCard::write_blocking(sector, &buf);

    Ok(())
}

pub fn led_set(ctx: &mut ActionContext) -> Result<()> {
    let bits = if let Some(bits) = ctx.tokens.next() {
        !u8_slice_bits_to_u32(bits)
        // note: inverted since '0' is 'on'
    } else {
        ctx.printer.p(b"which leds");
        ctx.printer.nlc(2);
        return Err(Error::WhichLeds);
    };

    Leds::set(bits);

    Ok(())
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "actions return Result for consistency"
)]
pub fn help(ctx: &mut ActionContext, help: &[u8]) -> Result<()> {
    ctx.printer.p(help);

    Ok(())
}

pub fn new_object(ctx: &mut ActionContext) -> Result<()> {
    // get object name
    let Some(object_name) = ctx.tokens.next() else {
        ctx.printer.p(b"what object name");
        ctx.printer.nlc(2);
        return Err(Error::WhatObjectName);
    };

    if ctx.world.objects.iter().any(|x| x.name == object_name) {
        ctx.printer.p(b"object already exists");
        ctx.printer.nlc(2);
        return Err(Error::ObjectAlreadyExists);
    }

    let object_id = {
        let object_id = ctx.world.objects.len();
        ctx.world.objects.push(Object {
            name: Name::from(object_name),
        });
        object_id
    };

    ctx.world.entities[ctx.entity].objects.push(object_id);

    Ok(())
}

pub fn new_location(ctx: &mut ActionContext) -> Result<()> {
    let Some(to_link_name) = ctx.tokens.next() else {
        ctx.printer.p(b"what link name");
        ctx.printer.nlc(2);
        return Err(Error::WhatToLinkName);
    };

    let Some(back_link_name) = ctx.tokens.next() else {
        ctx.printer.p(b"what back link name");
        ctx.printer.nlc(2);
        return Err(Error::WhatBackLinkName);
    };

    let Some(new_location_name) = ctx.tokens.next() else {
        ctx.printer.p(b"what new location name");
        ctx.printer.nlc(2);
        return Err(Error::WhatNewLocationName);
    };

    if ctx
        .world
        .locations
        .iter()
        .any(|x| x.name == new_location_name)
    {
        ctx.printer.p(b"location already exists");
        ctx.printer.nlc(2);
        return Err(Error::LocationAlreadyExists);
    }

    let from_location_id = ctx.world.entities[ctx.entity].location;

    let to_link_name_id = find_or_add_link(ctx.world, to_link_name);

    // check if link is already used
    if ctx.world.locations[from_location_id]
        .links
        .iter()
        .any(|x| x.link_name == to_link_name_id)
    {
        ctx.printer.p(b"link from this location already exists");
        ctx.printer.nlc(2);
        return Err(Error::LinkFromLocationAlreadyExists);
    }

    let back_link_name_id = find_or_add_link(ctx.world, back_link_name);

    // add location and link it back to from location
    let new_location_id = ctx.world.locations.len();
    ctx.world.locations.push(Location {
        name: Name::from(new_location_name),
        note: Note::default(),
        links: vec![Link {
            link_name: back_link_name_id,
            location: from_location_id,
        }],
        objects: vec![],
        entities: vec![],
    });

    ctx.world.locations[from_location_id].links.push(Link {
        link_name: to_link_name_id,
        location: new_location_id,
    });

    Ok(())
}

pub fn new_entity(ctx: &mut ActionContext) -> Result<()> {
    // get object name
    let Some(entity_name) = ctx.tokens.next() else {
        ctx.printer.p(b"what entity name");
        ctx.printer.nlc(2);
        return Err(Error::WhatEntityName);
    };

    if ctx.world.entities.iter().any(|x| x.name == entity_name) {
        ctx.printer.p(b"entity already exists");
        ctx.printer.nlc(2);
        return Err(Error::EntityAlreadyExists);
    }

    let location_id = ctx.world.entities[ctx.entity].location;
    let entity_id = ctx.world.entities.len();
    ctx.world.entities.push(Entity {
        name: Name::from(entity_name),
        location: location_id,
        objects: vec![],
        messages: vec![],
    });
    ctx.world.locations[location_id].entities.push(entity_id);

    Ok(())
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "actions return Result for consistency"
)]
pub fn set_location_note(ctx: &mut ActionContext) -> Result<()> {
    ctx.world.locations[ctx.world.entities[ctx.entity].location].note =
        Note::from(ctx.tokens.rest());

    Ok(())
}

pub fn say(ctx: &mut ActionContext) -> Result<()> {
    let say = ctx.tokens.rest();
    if say.is_empty() {
        ctx.printer.p(b"say what");
        ctx.printer.nlc(2);
        return Err(Error::SayWhat);
    }

    let entity = &ctx.world.entities[ctx.entity];
    send_message_to_entities_in_location(
        ctx.world,
        entity.location,
        &[ctx.entity],
        Message::from_parts(&[&entity.name, b" says ", say]),
    );

    Ok(())
}

pub fn tell(ctx: &mut ActionContext) -> Result<()> {
    let Some(to_name) = ctx.tokens.next() else {
        ctx.printer.p(b"tell to whom");
        ctx.printer.nlc(2);
        return Err(Error::TellToWhom);
    };

    let tell = ctx.tokens.rest();
    if tell.is_empty() {
        ctx.printer.p(b"tell what");
        ctx.printer.nlc(2);
        return Err(Error::TellWhat);
    }

    let entity = &ctx.world.entities[ctx.entity];

    let Some(&to_entity_id) = ctx.world.locations[entity.location]
        .entities
        .iter()
        .find(|&&x| ctx.world.entities[x].name == to_name)
    else {
        ctx.printer.p(to_name);
        ctx.printer.p(b" not here");
        ctx.printer.nlc(2);
        return Err(Error::EntityNotHere);
    };

    let message = Message::from_parts(&[&entity.name, b" tells u ", tell]);
    ctx.world.entities[to_entity_id].messages.push(message);

    Ok(())
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "actions return Result for consistency"
)]
pub const fn wait(_ctx: &mut ActionContext) -> Result<()> {
    Ok(())
}

//
// utilities
//

fn find_object_in_entity_inventory(
    world: &World,
    entity: EntityId,
    object_name: &[u8],
) -> Option<(usize, ObjectId)> {
    world.entities[entity]
        .objects
        .iter()
        .enumerate()
        .find_map(|(index, &oid)| (world.objects[oid].name == object_name).then_some((index, oid)))
}

fn find_or_add_link(world: &mut World, link_name: &[u8]) -> LinkNameId {
    if let Some(id) = world.link_names.iter().position(|x| x.name == link_name) {
        return id;
    }

    let id = world.link_names.len();
    world.link_names.push(LinkName {
        name: Name::from(link_name),
    });

    id
}

fn send_message_to_entities_in_location(
    world: &mut World,
    location: LocationId,
    exclude_entities: &[EntityId],
    message: Message,
) {
    for &eid in &world.locations[location].entities {
        if !exclude_entities.contains(&eid) {
            world.entities[eid].messages.push(message);
        }
    }
}

fn send_message_to_entities(world: &mut World, entities: &[EntityId], message: Message) {
    for &eid in entities {
        world.entities[eid].messages.push(message);
    }
}
