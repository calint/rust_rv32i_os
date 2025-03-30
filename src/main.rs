#![no_std]
#![no_main]

static HELLO: &[u8] = b"welcome to adventure #5\r\n    type 'help'\r\n\r\n";

static ASCII_ART: &[u8] = b":                                  oOo.o.\r\n\
:         frameless osca          oOo.oOo\r\n\
:      __________________________  .oOo.\r\n\
:     O\\        -_   .. \\    ___ \\   ||\r\n\
:    O  \\                \\   \\ \\\\ \\ //\\\\\r\n\
:   o   /\\    risc-v      \\   \\|\\\\ \\\r\n\
:  .   //\\\\    fpga        \\   ||   \\\r\n\
:   .  \\\\/\\\\    rust        \\  \\_\\   \\\r\n\
:    .  \\\\//\\________________\\________\\\r\n\
:     .  \\/_/, \\\\\\--\\\\..\\\\ - /\\_____  /\r\n\
:      .  \\ \\ . \\\\\\__\\\\__\\\\./ / \\__/ /\r\n\
:       .  \\ \\ , \\    \\\\ ///./ ,/./ /\r\n\
:        .  \\ \\___\\ sticky notes / /\r\n\
:         .  \\/\\________________/ /\r\n\
:    ./\\.  . / /                 /\r\n\
:    /--\\   .\\/_________________/\r\n\
:         ___.                 .\r\n\
:        |o o|. . . . . . . . .\r\n\
:        /| |\\ . .\r\n\
:    ____       . .\r\n\
:   |O  O|       . .\r\n\
:   |_ -_|        . .\r\n\
:    /||\\\r\n\
:      ___\r\n\
:     /- -\\\r\n\
:    /\\_-_/\\\r\n\
:      | |\r\n\
\r\n";

static HELP:&[u8]=b"\r\ncommand:\r\n  go <exit>: go\r\n  n: go north\r\n  e: go east\r\n  s: go south\r\n  w: go west\r\n  i: display inventory\r\n  t <object>: take object\r\n  d <object>: drop object\r\n  g <object> <entity>: give object to entity\r\n  say <what>: say to all in location\r\n  tell <whom> <what>: tells entity in location\r\n  sds: SD card status\r\n  sdr <sector>: read sector from SD card\r\n  sdw <sector> <text>: write sector to SD card\r\n  mi: memory info\r\n  led <decimal for bits (0 is on)>: turn on/off leds\r\n  no <object name>: new object into current inventory\r\n  nl <to link> <back link> <new location name>: new linked location\r\n  help: this message\r\n\r\n";

static CREATION: &[u8] = br#"nln todo: find an exit
nl none back office
go none
no notebook
d notebook
no lighter
d lighter
nl west east kitchen
nl east west bathroom
no mirror
ne u
go back
wait
"#;

extern crate alloc;

mod lib {
    pub mod api;
    pub mod api_unsafe;
    pub mod constants;
    pub mod cursor_buffer;
    pub mod global_allocator;
}
mod model;

use alloc::vec;
use core::arch::global_asm;
use core::panic::PanicInfo;
use lib::api::*;
use lib::api_unsafe::*;
use lib::cursor_buffer::*;
use lib::global_allocator::*;
use model::*;

const COMMAND_BUFFER_SIZE: usize = 80;

const CHAR_BACKSPACE: u8 = 0x7f;
const CHAR_CARRIAGE_RETURN: u8 = 0xd;
const CHAR_ESCAPE: u8 = 0x1b;

type CommandBuffer = CursorBuffer<COMMAND_BUFFER_SIZE, u8>;
type CommandBufferIterator<'a> = CursorBufferIterator<'a, COMMAND_BUFFER_SIZE, u8, fn(&u8) -> bool>;

// setup stack and jump to 'run()'
global_asm!(include_str!("startup.s"));

#[unsafe(no_mangle)]
pub extern "C" fn run() -> ! {
    led_set(0b0000); // turn all leds on

    global_allocator_init(memory_end() as usize);

    let mut world = World {
        entities: vec![Entity {
            name: Name::from(b"me"),
            location: 0,
            objects: vec![],
            messages: vec![],
        }],
        locations: vec![Location {
            name: Name::from(b"roome"),
            note: Note::new(),
            links: vec![],
            objects: vec![],
            entities: vec![0],
        }],
        objects: vec![],
        links: vec![],
    };

    execute_creation(&mut world, 0);

    uart_send_bytes(ASCII_ART);
    uart_send_bytes(HELLO);

    loop {
        for entity_id in 0..world.entities.len() {
            action_look(&mut world, entity_id);
            uart_send_cstr(&world.entities[entity_id].name.data);
            uart_send_bytes(b" > ");
            let mut command_buffer = CommandBuffer::new();
            input(&mut command_buffer);
            uart_send_bytes(b"\r\n");
            handle_input(&mut world, entity_id, &command_buffer);
        }
    }
}

fn handle_input(world: &mut World, entity_id: EntityId, command_buffer: &CommandBuffer) {
    let mut it: CommandBufferIterator = command_buffer.iter_words(|x| x.is_ascii_whitespace());
    match it.next() {
        Some(b"go") => action_go(world, entity_id, &mut it),
        Some(b"n") => action_go_named_link(world, entity_id, b"north"),
        Some(b"e") => action_go_named_link(world, entity_id, b"east"),
        Some(b"s") => action_go_named_link(world, entity_id, b"south"),
        Some(b"w") => action_go_named_link(world, entity_id, b"west"),
        Some(b"i") => action_inventory(world, entity_id),
        Some(b"t") => action_take(world, entity_id, &mut it),
        Some(b"d") => action_drop(world, entity_id, &mut it),
        Some(b"g") => action_give(world, entity_id, &mut it),
        Some(b"sds") => action_sdcard_status(),
        Some(b"sdr") => action_sdcard_read(&mut it),
        Some(b"sdw") => action_sdcard_write(&mut it),
        Some(b"mi") => action_memory_info(),
        Some(b"led") => action_led_set(&mut it),
        Some(b"help") => action_help(),
        Some(b"no") => action_new_object(world, entity_id, &mut it),
        Some(b"nl") => action_new_location(world, entity_id, &mut it),
        Some(b"nln") => action_set_location_note(world, entity_id, &mut it),
        Some(b"ne") => action_new_entity(world, entity_id, &mut it),
        Some(b"say") => action_say(world, entity_id, &mut it),
        Some(b"tell") => action_tell(world, entity_id, &mut it),
        Some(b"wait") => action_wait(),
        _ => uart_send_bytes(b"not understood\r\n\r\n"),
    }
}

fn action_look(world: &mut World, entity_id: EntityId) {
    {
        let entity = &mut world.entities[entity_id];
        let location = &world.locations[entity.location];

        let messages = &entity.messages;
        messages.iter().for_each(|x| {
            uart_send_cstr(&x.data);
            uart_send_bytes(b"\r\n");
        });

        // clear messages after displayed
        entity.messages.clear();

        uart_send_bytes(b"u r in ");
        uart_send_cstr(&location.name.data);

        uart_send_bytes(b"\r\nu c: ");
        let mut i = 0;
        for &oid in &location.objects {
            if i != 0 {
                uart_send_bytes(b", ");
            }
            i += 1;
            uart_send_cstr(&world.objects[oid].name.data);
        }
        if i == 0 {
            uart_send_bytes(b"nothing");
        }
        uart_send_bytes(b"\r\n");

        let mut i = 0;
        for &eid in &location.entities {
            if eid != entity_id {
                if i != 0 {
                    uart_send_bytes(b", ");
                }
                uart_send_cstr(&world.entities[eid].name.data);
                i += 1;
            }
        }
        if i > 0 {
            uart_send_bytes(b" is here\r\n");
        }

        uart_send_bytes(b"exits: ");
        let mut i = 0;
        for lid in &location.links {
            if i != 0 {
                uart_send_bytes(b", ");
            }
            i += 1;
            uart_send_cstr(&world.links[lid.link].name.data);
        }
        if i == 0 {
            uart_send_bytes(b"none");
        }
        uart_send_bytes(b"\r\n");

        if !location.note.is_empty() {
            uart_send_cstr(&location.note.data);
            uart_send_bytes(b"\r\n");
        }
    }
}

fn action_go(world: &mut World, entity_id: EntityId, it: &mut CommandBufferIterator) {
    let named_link = match it.next() {
        Some(name) => name,
        None => {
            uart_send_bytes(b"go where");
            return;
        }
    };

    action_go_named_link(world, entity_id, named_link);
}

fn action_go_named_link(world: &mut World, entity_id: EntityId, link_name: &[u8]) {
    // find link id
    let link_id = match world.links.iter().position(|x| x.name.equals(link_name)) {
        Some(id) => id,
        None => {
            uart_send_bytes(b"no such exit\r\n\r\n");
            return;
        }
    };

    // move entity
    let (from_location_id, to_location_id) = {
        let entity = &mut world.entities[entity_id];
        let from_location_id = entity.location;
        let from_location = &mut world.locations[from_location_id];

        // find "to" location id
        let to_location_id = match from_location.links.iter().find(|x| x.link == link_id) {
            Some(lnk) => lnk.location,
            None => {
                uart_send_bytes(b"can't go there\r\n\r\n");
                return;
            }
        };

        // remove entity from old location
        if let Some(pos) = from_location.entities.iter().position(|&x| x == entity_id) {
            from_location.entities.remove(pos);
        }

        // add entity to new location
        world.locations[to_location_id].entities.push(entity_id);

        // update entity location
        entity.location = to_location_id;

        (from_location_id, to_location_id)
    };

    // send message to entities in 'from_location' that entity has left
    send_message_to_location_entities(
        world,
        from_location_id,
        &[entity_id],
        EntityMessage::from(&[
            &world.entities[entity_id].name.data,
            b" left to ",
            link_name,
        ]),
    );

    // find link name that leads from 'to_location_id' to 'from_location_id'
    // note: assumes links are bi-directional thus unwrap
    let link_id = world.locations[to_location_id]
        .links
        .iter()
        .find_map(|x| {
            if x.location == from_location_id {
                Some(x.link)
            } else {
                None
            }
        })
        .unwrap();

    // send message to entities in 'to_location' that entity has arrived
    send_message_to_location_entities(
        world,
        to_location_id,
        &[entity_id],
        EntityMessage::from(&[
            &world.entities[entity_id].name.data,
            b" arrived from ",
            &world.links[link_id].name.data,
        ]),
    );
}

fn action_inventory(world: &World, entity_id: EntityId) {
    let entity = &world.entities[entity_id];
    uart_send_bytes(b"u have: ");
    let mut i = 0;
    for &oid in &entity.objects {
        if i != 0 {
            uart_send_bytes(b", ");
        }
        i += 1;
        uart_send_cstr(&world.objects[oid].name.data);
    }
    if i == 0 {
        uart_send_bytes(b"nothing");
    }
    uart_send_bytes(b"\r\n\r\n");
}

fn action_take(world: &mut World, entity_id: EntityId, it: &mut CommandBufferIterator) {
    // get object name
    let object_name = match it.next() {
        Some(name) => name,
        None => {
            uart_send_bytes(b"take what\r\n\r\n");
            return;
        }
    };

    {
        let entity = &mut world.entities[entity_id];
        let location = &mut world.locations[entity.location];

        // find object id and index in list
        let (object_index, object_id) = match location
            .objects
            .iter()
            .enumerate()
            .find(|&(_, &oid)| world.objects[oid].name.equals(object_name))
        {
            Some((index, &oid)) => (index, oid),
            None => {
                uart_send_bytes(object_name);
                uart_send_bytes(b" is not here\r\n\r\n");
                return;
            }
        };

        // remove object from location
        location.objects.remove(object_index);

        // add object to entity
        entity.objects.push(object_id);
    }

    // send message
    {
        let entity = &world.entities[entity_id];
        send_message_to_location_entities(
            world,
            entity.location,
            &[entity_id],
            EntityMessage::from(&[&entity.name.data, b" took ", object_name]),
        );
    }
}

fn action_drop(world: &mut World, entity_id: EntityId, it: &mut CommandBufferIterator) {
    let object_name = match it.next() {
        Some(name) => name,
        None => {
            uart_send_bytes(b"drop what\r\n\r\n");
            return;
        }
    };

    {
        let (object_index, object_id) =
            match find_object_in_entity_inventory(world, entity_id, object_name) {
                Some(result) => result,
                None => {
                    uart_send_bytes(b"don't have ");
                    uart_send_bytes(object_name);
                    uart_send_bytes(b"\r\n\r\n");
                    return;
                }
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
        send_message_to_location_entities(
            world,
            entity.location,
            &[entity_id],
            EntityMessage::from(&[&entity.name.data, b" dropped ", object_name]),
        );
    }
}

fn action_give(world: &mut World, entity_id: EntityId, it: &mut CommandBufferIterator) {
    // get object name
    let object_name = match it.next() {
        Some(name) => name,
        None => {
            uart_send_bytes(b"give what\r\n\r\n");
            return;
        }
    };

    // get entity name
    let to_entity_name = match it.next() {
        Some(name) => name,
        None => {
            uart_send_bytes(b"give to whom\r\n\r\n");
            return;
        }
    };

    // find "to" entity
    let to_entity_id = match world.locations[world.entities[entity_id].location]
        .entities
        .iter()
        .find(|&&x| world.entities[x].name.equals(to_entity_name))
    {
        Some(&id) => id,
        None => {
            uart_send_bytes(to_entity_name);
            uart_send_bytes(b" not here\r\n\r\n");
            return;
        }
    };

    let (object_index, object_id) =
        match find_object_in_entity_inventory(world, entity_id, object_name) {
            Some(result) => result,
            None => {
                uart_send_bytes(b"don't have ");
                uart_send_bytes(object_name);
                uart_send_bytes(b"\r\n\r\n");
                return;
            }
        };

    // remove object from entity
    world.entities[entity_id].objects.remove(object_index);

    // add object to "to" entity
    world.entities[to_entity_id].objects.push(object_id);

    // send messages
    send_message_to_location_entities(
        world,
        world.entities[entity_id].location,
        &[to_entity_id],
        EntityMessage::from(&[
            &world.entities[entity_id].name.data,
            b" gave ",
            &world.objects[object_id].name.data,
            b" to ",
            &world.entities[to_entity_id].name.data,
        ]),
    );

    send_message_to_entities(
        world,
        &[to_entity_id],
        EntityMessage::from(&[
            &world.entities[entity_id].name.data,
            b" gave u ",
            &world.objects[object_id].name.data,
        ]),
    );
}

fn action_memory_info() {
    uart_send_bytes(b"   heap start: ");
    uart_send_hex_u32(memory_heap_start(), true);
    uart_send_bytes(b"\r\nstack pointer: ");
    uart_send_hex_u32(memory_stack_pointer(), true);
    uart_send_bytes(b"\r\n   memory end: ");
    uart_send_hex_u32(memory_end(), true);
    uart_send_bytes(b"\r\n\r\nheap blocks:\r\n");
    global_allocator_debug_block_list();
    uart_send_bytes(b"\r\n");
}

fn action_sdcard_status() {
    uart_send_bytes(b"SDCARD_STATUS: 0x");
    uart_send_hex_u32(sdcard_status() as u32, true);
    uart_send_bytes(b"\r\n\r\n");
}

fn action_sdcard_read(it: &mut CommandBufferIterator) {
    let sector = match it.next() {
        Some(sector) => u8_slice_to_u32(sector),
        None => {
            uart_send_bytes(b"what sector\r\n\r\n");
            return;
        }
    };

    let mut buf = [0; 512];
    sdcard_read_blocking(sector, &mut buf);
    buf.iter().for_each(|&x| uart_send_byte(x));
    uart_send_bytes(b"\r\n\r\n");
}

fn action_sdcard_write(it: &mut CommandBufferIterator) {
    let sector = match it.next() {
        Some(sector) => u8_slice_to_u32(sector),
        None => {
            uart_send_bytes(b"what sector\r\n\r\n");
            return;
        }
    };

    let rest = it.rest();
    let len = rest.len().min(512);
    let mut buf = [0u8; 512];
    buf[..len].copy_from_slice(&rest[..len]);
    sdcard_write_blocking(sector, &buf);
}

fn action_led_set(it: &mut CommandBufferIterator) {
    let bits = match it.next() {
        Some(bits) => u8_slice_to_u32(bits),
        None => {
            uart_send_bytes(b"which leds in bits 0 being on\r\n\r\n");
            return;
        }
    };

    led_set(bits as u8);
}

fn action_help() {
    uart_send_bytes(HELP);
}

fn action_new_object(world: &mut World, entity_id: EntityId, it: &mut CommandBufferIterator) {
    // get object name
    let object_name = match it.next() {
        Some(name) => name,
        None => {
            uart_send_bytes(b"what object name\r\n\r\n");
            return;
        }
    };

    if world.objects.iter().any(|x| x.name.equals(object_name)) {
        uart_send_bytes(b"object already exists\r\n\r\n");
        return;
    }

    let object_id = world.add_object(object_name);

    world.entities[entity_id].objects.push(object_id);
}

fn action_new_location(world: &mut World, entity_id: EntityId, it: &mut CommandBufferIterator) {
    let to_link_name = match it.next() {
        Some(name) => name,
        None => {
            uart_send_bytes(b"what to link name\r\n\r\n");
            return;
        }
    };

    let back_link_name = match it.next() {
        Some(name) => name,
        None => {
            uart_send_bytes(b"what back link name\r\n\r\n");
            return;
        }
    };

    let new_location_name = match it.next() {
        Some(name) => name,
        None => {
            uart_send_bytes(b"what new location name\r\n\r\n");
            return;
        }
    };

    if world
        .locations
        .iter()
        .any(|x| x.name.equals(new_location_name))
    {
        uart_send_bytes(b"location already exists\r\n\r\n");
        return;
    }

    let from_location_id = world.entities[entity_id].location;

    let to_link_id = world.find_or_add_link(to_link_name);

    // check if link is already used
    if world.locations[from_location_id]
        .links
        .iter()
        .any(|x| x.link == to_link_id)
    {
        uart_send_bytes(b"this location already uses link");
        return;
    }

    let back_link_id = world.find_or_add_link(back_link_name);

    // add location and link it back to from location
    let new_location_id = world.locations.len();
    world.locations.push(Location {
        name: Name::from(new_location_name),
        note: Note::new(),
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
}

fn action_new_entity(world: &mut World, entity_id: EntityId, it: &mut CommandBufferIterator) {
    // get object name
    let entity_name = match it.next() {
        Some(name) => name,
        None => {
            uart_send_bytes(b"what entity name\r\n\r\n");
            return;
        }
    };

    if world.entities.iter().any(|x| x.name.equals(entity_name)) {
        uart_send_bytes(b"entity already exists\r\n\r\n");
        return;
    }

    world.add_entity(entity_name, world.entities[entity_id].location);
}

fn action_set_location_note(
    world: &mut World,
    entity_id: EntityId,
    it: &mut CommandBufferIterator,
) {
    world.locations[world.entities[entity_id].location].note = Note::from(it.rest());
}

fn action_say(world: &mut World, entity_id: EntityId, it: &mut CommandBufferIterator) {
    let say = it.rest();
    if say.is_empty() {
        uart_send_bytes(b"say what");
        return;
    }

    let entity = &world.entities[entity_id];
    send_message_to_location_entities(
        world,
        entity.location,
        &[entity_id],
        EntityMessage::from(&[&entity.name.data, b" says ", say]),
    );
}

fn action_tell(world: &mut World, entity_id: EntityId, it: &mut CommandBufferIterator) {
    let to_name = match it.next() {
        Some(name) => name,
        None => {
            uart_send_bytes(b"tell to whom\r\n");
            return;
        }
    };

    let tell = it.rest();
    if tell.is_empty() {
        uart_send_bytes(b"tell what\r\n");
        return;
    };

    let entity = &world.entities[entity_id];

    let to_entity_id = match world.locations[entity.location]
        .entities
        .iter()
        .find(|&&x| world.entities[x].name.equals(to_name))
    {
        Some(&id) => id,
        None => {
            uart_send_bytes(b"not here\r\n");
            return;
        }
    };

    let message = EntityMessage::from(&[&entity.name.data, b" tells u ", tell]);
    world.entities[to_entity_id].messages.push(message);
}

fn action_wait() {}

fn input(command_buffer: &mut CommandBuffer) {
    enum InputState {
        Normal,
        Escape,
        EscapeBracket,
    }

    let mut state = InputState::Normal;
    let mut escape_sequence_parameter = 0;

    command_buffer.reset();

    loop {
        let ch = uart_read_byte();
        led_set(!ch);

        match state {
            InputState::Normal => {
                if ch == CHAR_ESCAPE {
                    state = InputState::Escape;
                } else if ch == CHAR_BACKSPACE {
                    if command_buffer.backspace() {
                        uart_send_byte(ch);
                        command_buffer.for_each_from_cursor(uart_send_byte);
                        uart_send_byte(b' ');
                        uart_send_move_back(command_buffer.elements_after_cursor_count() + 1);
                    }
                } else if ch == CHAR_CARRIAGE_RETURN || command_buffer.is_full() {
                    return;
                } else if command_buffer.insert(ch) {
                    uart_send_byte(ch);
                    command_buffer.for_each_from_cursor(uart_send_byte);
                    uart_send_move_back(command_buffer.elements_after_cursor_count());
                }
            }
            InputState::Escape => {
                if ch == b'[' {
                    state = InputState::EscapeBracket;
                } else {
                    state = InputState::Normal;
                }
            }
            InputState::EscapeBracket => {
                if ch.is_ascii_digit() {
                    escape_sequence_parameter = escape_sequence_parameter * 10 + (ch - b'0');
                } else {
                    match ch {
                        b'D' => {
                            // arrow left
                            if command_buffer.move_cursor_left() {
                                uart_send_bytes(b"\x1B[D");
                            }
                        }
                        b'C' => {
                            // arrow right
                            if command_buffer.move_cursor_right() {
                                uart_send_bytes(b"\x1B[C");
                            }
                        }
                        b'~' => {
                            // delete
                            if escape_sequence_parameter == 3 {
                                // delete key
                                command_buffer.del();
                                command_buffer.for_each_from_cursor(uart_send_byte);
                                uart_send_byte(b' ');
                                uart_send_move_back(
                                    command_buffer.elements_after_cursor_count() + 1,
                                    // note: +1 to compensate for the ' ' done to erase last character
                                );
                            }
                        }
                        _ => {}
                    }
                    state = InputState::Normal;
                    escape_sequence_parameter = 0;
                }
            }
        }
    }
}

fn execute_creation(world: &mut World, entity_id: EntityId) {
    for line in CREATION.split(|&c| c == b'\n').filter(|x| !x.is_empty()) {
        let mut command_buffer = CommandBuffer::new();
        for &byte in line {
            if !command_buffer.insert(byte) {
                break;
            }
        }

        handle_input(world, entity_id, &command_buffer);

        // clear messages on all entities in case input generated messages
        world.entities.iter_mut().for_each(|x| x.messages.clear());
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart_send_bytes(b"PANIC!!!");
    loop {}
}
