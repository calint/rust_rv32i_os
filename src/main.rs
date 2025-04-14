#![no_std]
#![no_main]

static HELLO: &[u8] = b"welcome to adventure #5\r\n    type 'help'\r\n\r\n";

static ASCII_ART: &[u8] = b"\x20                                  oOo.o.\r\n\
\x20         frameless osca          oOo.oOo\r\n\
\x20      __________________________  .oOo.\r\n\
\x20     O\\        -_   .. \\    ___ \\   ||\r\n\
\x20    O  \\                \\   \\ \\\\ \\ //\\\\\r\n\
\x20   o   /\\    risc-v      \\   \\|\\\\ \\\r\n\
\x20  .   //\\\\    fpga        \\   ||   \\\r\n\
\x20   .  \\\\/\\\\    rust        \\  \\_\\   \\\r\n\
\x20    .  \\\\//\\________________\\________\\\r\n\
\x20     .  \\/_/, \\\\\\--\\\\..\\\\ - /\\_____  /\r\n\
\x20      .  \\ \\ . \\\\\\__\\\\__\\\\./ / \\__/ /\r\n\
\x20       .  \\ \\ , \\    \\\\ ///./ ,/./ /\r\n\
\x20        .  \\ \\___\\ sticky notes / /\r\n\
\x20         .  \\/\\________________/ /\r\n\
\x20    ./\\.  . / /                 /\r\n\
\x20    /--\\   .\\/_________________/\r\n\
\x20         ___.                 .\r\n\
\x20        |o o|. . . . . . . . .\r\n\
\x20        /| |\\ . .\r\n\
\x20    ____       . .\r\n\
\x20   |O  O|       . .\r\n\
\x20   |_ -_|        . .\r\n\
\x20    /||\\\r\n\
\x20      ___\r\n\
\x20     /- -\\\r\n\
\x20    /\\_-_/\\\r\n\
\x20      | |\r\n\
\r\n";

static HELP:&[u8]=b"\r\ncommand:\r\n  go <exit>: go\r\n  n: go north\r\n  e: go east\r\n  s: go south\r\n  w: go west\r\n  i: display inventory\r\n  t <object>: take object\r\n  d <object>: drop object\r\n  g <object> <entity>: give object to entity\r\n  say <what>: say to all in location\r\n  tell <whom> <what>: tells entity in location\r\n  sds: SD card status\r\n  sdr <sector>: read sector from SD card\r\n  sdw <sector> <text>: write sector to SD card\r\n  mi: memory info\r\n  led <decimal for bits (0 is on)>: turn on/off leds\r\n  no <object name>: new object into current inventory\r\n  nl <to link> <back link> <new location name>: new linked location\r\n  help: this message\r\n\r\n";

static CREATION: &[u8] = b"nln todo: find an exit
nl none back office
go none
no notebook
d notebook
no lighter
d lighter
nl west east kitchen
nl east west bathroom
no mirror
go back
ne me";

mod lib {
    pub mod api;
    pub mod api_unsafe;
    pub mod constants;
    pub mod cursor_buffer;
    pub mod fixed_size_string;
    pub mod global_allocator;
}
mod model;

extern crate alloc;

use alloc::vec;
use core::arch::global_asm;
use core::panic::PanicInfo;
use lib::api::{
    memory_end, memory_heap_start, u8_slice_to_u32, uart_send_bytes, uart_send_hex_u32,
    uart_send_move_back,
};
use lib::api_unsafe::{
    SDCARD_SECTOR_SIZE_BYTES, led_set, memory_stack_pointer, sdcard_read_blocking, sdcard_status,
    sdcard_write_blocking, uart_read_byte, uart_send_byte,
};
use lib::cursor_buffer::{CursorBuffer, CursorBufferIterator};
use lib::global_allocator::GlobalAllocator;
use model::{Entity, EntityId, EntityMessage, Location, LocationLink, Name, Note, World};

const COMMAND_BUFFER_SIZE: usize = 80;

const CHAR_BACKSPACE: u8 = 0x7f;
const CHAR_CARRIAGE_RETURN: u8 = 0xd;
const CHAR_ESCAPE: u8 = 0x1b;

type CommandBuffer = CursorBuffer<COMMAND_BUFFER_SIZE, u8>;
type CommandBufferIterator<'a> = CursorBufferIterator<'a, COMMAND_BUFFER_SIZE, u8, fn(&u8) -> bool>;

// setup bss section, stack and jump to 'run()'
global_asm!(include_str!("startup.s"));

/// # Panics
///
/// Will panic if look fails.
#[unsafe(no_mangle)]
pub extern "C" fn run() -> ! {
    led_set(0b0000); // turn all leds on

    GlobalAllocator::init(memory_end() as usize);

    let mut world = create_world();

    uart_send_bytes(ASCII_ART);
    uart_send_bytes(HELLO);

    loop {
        for entity_id in 0..world.entities.len() {
            assert!(action_look(&mut world, entity_id).is_ok(), "cannot look");
            loop {
                // loop until action succeeded
                uart_send_bytes(&world.entities[entity_id].name);
                uart_send_bytes(b" > ");
                let mut command_buffer = CommandBuffer::new();
                input(&mut command_buffer);
                uart_send_bytes(b"\r\n");
                if handle_input(&mut world, entity_id, &command_buffer, true).is_ok() {
                    break;
                }
            }
        }
    }
}

fn handle_input(
    world: &mut World,
    entity_id: EntityId,
    command_buffer: &CommandBuffer,
    new_line_after_result: bool,
) -> Result<()> {
    let mut it: CommandBufferIterator = command_buffer.iter_words(u8::is_ascii_whitespace);
    match it.next() {
        Some(b"go") => action_go(world, entity_id, &mut it)?,
        Some(b"n") => action_go_named_link(world, entity_id, b"north")?,
        Some(b"e") => action_go_named_link(world, entity_id, b"east")?,
        Some(b"s") => action_go_named_link(world, entity_id, b"south")?,
        Some(b"w") => action_go_named_link(world, entity_id, b"west")?,
        Some(b"i") => action_inventory(world, entity_id)?,
        Some(b"t") => action_take(world, entity_id, &mut it)?,
        Some(b"d") => action_drop(world, entity_id, &mut it)?,
        Some(b"g") => action_give(world, entity_id, &mut it)?,
        Some(b"sds") => action_sdcard_status()?,
        Some(b"sdr") => action_sdcard_read(&mut it)?,
        Some(b"sdw") => action_sdcard_write(&mut it)?,
        Some(b"mi") => action_memory_info()?,
        Some(b"led") => action_led_set(&mut it)?,
        Some(b"help") => action_help()?,
        Some(b"no") => action_new_object(world, entity_id, &mut it)?,
        Some(b"nl") => action_new_location(world, entity_id, &mut it)?,
        Some(b"nln") => action_set_location_note(world, entity_id, &mut it)?,
        Some(b"ne") => action_new_entity(world, entity_id, &mut it)?,
        Some(b"say") => action_say(world, entity_id, &mut it)?,
        Some(b"tell") => action_tell(world, entity_id, &mut it)?,
        Some(b"wait") => action_wait()?,
        _ => {
            uart_send_bytes(b"not understood\r\n\r\n");
            return Err(ActionFailed::InvalidCommand);
        }
    }

    if new_line_after_result {
        uart_send_bytes(b"\r\n");
    }

    Ok(())
}

type Result<T> = core::result::Result<T, ActionFailed>;

enum ActionFailed {
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
    EntityNotInLocation,
    WhatSector,
    WhichLeds,
    WhatObjectName,
    ObjectAlreadyExists,
    WhatToLinkName,
    WhatBackLinkName,
    WhatNewLocationName,
    LocationAlreadyExists,
    LinkFromLocationAlreadyExists,
    EntityAlreadyExists,
    WhatEntityName,
    SayWhat,
    TellToWhom,
    TellWhat,
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "All actions return Result for consistency."
)]
fn action_look(world: &mut World, entity_id: EntityId) -> Result<()> {
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

fn action_go(world: &mut World, entity_id: EntityId, it: &mut CommandBufferIterator) -> Result<()> {
    let Some(named_link) = it.next() else {
        uart_send_bytes(b"go where\r\n\r\n");
        return Err(ActionFailed::GoWhere);
    };

    action_go_named_link(world, entity_id, named_link)
}

fn action_go_named_link(world: &mut World, entity_id: EntityId, link_name: &[u8]) -> Result<()> {
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
        let Some(pos) = from_location.entities.iter().position(|&x| x == entity_id) else {
            panic!();
        };

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
    let Some(back_link_id) = world.locations[to_location_id]
        .links
        .iter()
        .find_map(|x| (x.location == from_location_id).then_some(x.link))
    else {
        panic!();
    };

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
    reason = "All actions return Result for consistency."
)]
fn action_inventory(world: &World, entity_id: EntityId) -> Result<()> {
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

fn action_take(
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

fn action_drop(
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

fn action_give(
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
        return Err(ActionFailed::EntityNotInLocation);
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
    reason = "All actions return Result for consistency."
)]
fn action_memory_info() -> Result<()> {
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

#[expect(clippy::cast_sign_loss, reason = "Intended behavior.")]
#[allow(
    clippy::unnecessary_wraps,
    reason = "All actions return Result for consistency."
)]
fn action_sdcard_status() -> Result<()> {
    uart_send_bytes(b"SDCARD_STATUS: 0x");
    uart_send_hex_u32(sdcard_status() as u32, true);
    uart_send_bytes(b"\r\n");

    Ok(())
}

fn action_sdcard_read(it: &mut CommandBufferIterator) -> Result<()> {
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

fn action_sdcard_write(it: &mut CommandBufferIterator) -> Result<()> {
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

#[expect(clippy::cast_possible_truncation, reason = "Intended behavior.")]
fn action_led_set(it: &mut CommandBufferIterator) -> Result<()> {
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
    reason = "All actions return Result for consistency."
)]
fn action_help() -> Result<()> {
    uart_send_bytes(HELP);

    Ok(())
}

fn action_new_object(
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

fn action_new_location(
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

fn action_new_entity(
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
    reason = "All actions return Result for consistency."
)]
fn action_set_location_note(
    world: &mut World,
    entity_id: EntityId,
    it: &mut CommandBufferIterator,
) -> Result<()> {
    world.locations[world.entities[entity_id].location].note = Note::from(it.rest());

    Ok(())
}

fn action_say(
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

fn action_tell(
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
        return Err(ActionFailed::EntityNotInLocation);
    };

    let message = EntityMessage::from_parts(&[&entity.name, b" tells u ", tell]);
    world.entities[to_entity_id].messages.push(message);

    Ok(())
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "All actions return Result for consistency."
)]
const fn action_wait() -> Result<()> {
    Ok(())
}

fn input(command_buffer: &mut CommandBuffer) {
    loop {
        let ch = uart_read_byte();
        led_set(!ch);

        match ch {
            CHAR_ESCAPE => input_escape_sequence(command_buffer),
            CHAR_BACKSPACE => input_backspace(command_buffer),
            CHAR_CARRIAGE_RETURN => return,
            _ if command_buffer.is_full() => return,
            _ => input_normal_char(command_buffer, ch),
        }
    }
}

fn input_escape_sequence(command_buffer: &mut CommandBuffer) {
    if uart_read_byte() != b'[' {
        return;
    }

    let mut parameter = 0;
    loop {
        let ch = uart_read_byte();
        if ch.is_ascii_digit() {
            parameter = parameter * 10 + (ch - b'0');
        } else {
            match ch {
                b'D' => {
                    if command_buffer.move_cursor_left() {
                        uart_send_bytes(b"\x1B[D");
                    }
                }
                b'C' => {
                    if command_buffer.move_cursor_right() {
                        uart_send_bytes(b"\x1B[C");
                    }
                }
                b'~' => {
                    command_buffer.del();
                    command_buffer.for_each_from_cursor(uart_send_byte);
                    uart_send_byte(b' ');
                    uart_send_move_back(command_buffer.elements_after_cursor_count() + 1);
                    // note: +1 because of ' ' that erases the trailing character
                }
                _ => {}
            }
            return;
        }
    }
}

fn input_backspace(command_buffer: &mut CommandBuffer) {
    if command_buffer.backspace() {
        uart_send_byte(CHAR_BACKSPACE);
        command_buffer.for_each_from_cursor(uart_send_byte);
        uart_send_byte(b' ');
        uart_send_move_back(command_buffer.elements_after_cursor_count() + 1);
    }
}

fn input_normal_char(command_buffer: &mut CommandBuffer, ch: u8) {
    if command_buffer.insert(ch) {
        uart_send_byte(ch);
        command_buffer.for_each_from_cursor(uart_send_byte);
        uart_send_move_back(command_buffer.elements_after_cursor_count());
    }
}

fn create_world() -> World {
    let mut world = World {
        entities: vec![Entity {
            name: Name::from(b"u"),
            location: 0,
            objects: vec![],
            messages: vec![],
        }],
        locations: vec![Location {
            name: Name::from(b"roome"),
            note: Note::default(),
            links: vec![],
            objects: vec![],
            entities: vec![0],
        }],
        objects: vec![],
        links: vec![],
    };

    for line in CREATION.split(|&x| x == b'\n') {
        let mut command_buffer = CommandBuffer::new();
        for &byte in line {
            assert!(command_buffer.insert(byte), "command to large");
        }

        assert!(
            handle_input(&mut world, 0, &command_buffer, false).is_ok(),
            "error creating world"
        );

        // clear messages on all entities in case input generated messages
        world.entities.iter_mut().for_each(|x| x.messages.clear());
    }

    world
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart_send_bytes(b"PANIC!!!");
    led_set(0b0000);
    loop {}
}
