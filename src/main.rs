#![no_std]
#![no_main]
#![feature(allocator_api)]

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

static HELP:&[u8]=b"\r\ncommand:\r\n  n: go north\r\n  e: go east\r\n  s: go south\r\n  w: go west\r\n  i: display inventory\r\n  t <object>: take object\r\n  d <object>: drop object\r\n  g <object> <entity>: give object to entity\r\n  sds: SD card status\r\n  sdr <sector>: read sector from SD card\r\n  sdw <sector> <text>: write sector to SD card\r\n  mi: memory info\r\n  led <decimal for bits (0 is on)>: turn on/off leds\r\n  help: this message\r\n\r\n";

mod lib {
    pub mod api;
    pub mod api_unsafe;
    pub mod constants;
    pub mod cursor_buffer;
    // pub mod fixed_size_list;
    // pub mod gen_list;
    pub mod bump_allocator;
}

extern crate alloc;

use alloc::vec::Vec;
use core::arch::global_asm;
use core::panic::PanicInfo;
use lib::api::*;
use lib::api_unsafe::*;
use lib::bump_allocator::*;
use lib::cursor_buffer::*;

const COMMAND_BUFFER_SIZE: usize = 80;
const NAME_SIZE: usize = 32;

const CHAR_BACKSPACE: u8 = 0x7f;
const CHAR_CARRIAGE_RETURN: u8 = 0xd;
const CHAR_ESCAPE: u8 = 0x1b;

type LocationId = usize;
type LinkId = usize;
type EntityId = usize;
type ObjectId = usize;

type CommandBuffer = CursorBuffer<COMMAND_BUFFER_SIZE, u8>;
type CommandBufferIterator<'a> = CursorBufferIterator<'a, COMMAND_BUFFER_SIZE, u8, fn(&u8) -> bool>;

struct Name {
    data: [u8; NAME_SIZE],
}

impl Name {
    fn new() -> Self {
        Name {
            data: [0u8; NAME_SIZE],
        }
    }

    fn from(src: &[u8]) -> Self {
        let mut name = Name::new();
        let len = src.len().min(NAME_SIZE - 1);
        // note: -1 to enabled string terminator at the end of string
        name.data[..len].copy_from_slice(&src[..len]);
        name
    }

    fn equals(&self, compare_with: &[u8]) -> bool {
        if compare_with.len() >= NAME_SIZE {
            // note: >= to ensure the end of string terminator can be compared
            return false;
        }
        self.data.starts_with(compare_with) && self.data[compare_with.len()] == 0
    }
}

struct Object {
    name: Name,
}

struct Entity {
    name: Name,
    location: LocationId,
    objects: Vec<ObjectId>,
}

struct Link {
    name: Name,
}

struct LocationLink {
    link: LinkId,
    location: LocationId,
}

struct Location {
    name: Name,
    links: Vec<LocationLink>,
    objects: Vec<ObjectId>,
    entities: Vec<EntityId>,
}

struct World {
    objects: Vec<Object>,
    entities: Vec<Entity>,
    locations: Vec<Location>,
    links: Vec<Link>,
}

// setup stack and jump to 'run()'
global_asm!(include_str!("startup.s"));

#[unsafe(no_mangle)]
pub extern "C" fn run() -> ! {
    allocator_init();

    let mut world = World {
        objects: {
            let mut objects = Vec::new();
            objects.push(Object {
                name: Name::from(b"notebook"),
            });
            objects.push(Object {
                name: Name::from(b"mirror"),
            });
            objects.push(Object {
                name: Name::from(b"lighter"),
            });
            objects
        },
        entities: {
            let mut entities = Vec::new();
            entities.push(Entity {
                name: Name::from(b"me"),
                location: 0,
                objects: {
                    let mut list = Vec::new();
                    list.push(1);
                    list
                },
            });
            entities.push(Entity {
                name: Name::from(b"u"),
                location: 1,
                objects: Vec::new(),
            });
            entities
        },
        locations: {
            let mut locations = Vec::new();
            locations.push(Location {
                name: Name::from(b"roome"),
                links: {
                    let mut list = Vec::new();
                    list.push(LocationLink {
                        link: 0,
                        location: 1,
                    });
                    list.push(LocationLink {
                        link: 1,
                        location: 2,
                    });
                    list.push(LocationLink {
                        link: 3,
                        location: 3,
                    });
                    list
                },
                objects: Vec::new(),
                entities: {
                    let mut list = Vec::new();
                    list.push(0);
                    list
                },
            });
            locations.push(Location {
                name: Name::from(b"office"),
                links: {
                    let mut list = Vec::new();
                    list.push(LocationLink {
                        link: 2,
                        location: 0,
                    });
                    list
                },
                objects: {
                    let mut list = Vec::new();
                    list.push(0);
                    list.push(2);
                    list
                },
                entities: {
                    let mut list = Vec::new();
                    list.push(1);
                    list
                },
            });
            locations.push(Location {
                name: Name::from(b"bathroom"),
                links: Vec::new(),
                objects: Vec::new(),
                entities: Vec::new(),
            });
            locations.push(Location {
                name: Name::from(b"kitchen"),
                links: {
                    let mut list = Vec::new();
                    list.push(LocationLink {
                        link: 1,
                        location: 0,
                    });
                    list
                },
                objects: Vec::new(),
                entities: Vec::new(),
            });
            locations
        },
        links: {
            let mut links = Vec::new();
            links.push(Link {
                name: Name::from(b"north"),
            });
            links.push(Link {
                name: Name::from(b"east"),
            });
            links.push(Link {
                name: Name::from(b"south"),
            });
            links.push(Link {
                name: Name::from(b"west"),
            });
            links.push(Link {
                name: Name::from(b"up"),
            });
            links.push(Link {
                name: Name::from(b"down"),
            });
            links
        },
    };
    // let o1 = Box::new(Object { name: b"object1" });
    // let o2 = Box::new(Object { name: b"object2" });

    // uart_send_str(o1.name);
    // uart_send_str(o2.name);

    led_set(0b0000); // turn all leds on

    uart_send_str(ASCII_ART);
    uart_send_str(HELLO);

    loop {
        let entities_count = world.entities.len();
        for entity_id in 0..entities_count {
            let entity = match world.entities.get(entity_id) {
                Some(e) => e,
                None => continue,
            };
            action_look(&world, entity_id);
            uart_send_cstr(&entity.name.data);
            uart_send_str(b" > ");
            let mut command_buffer = CommandBuffer::new();
            input(&mut command_buffer);
            uart_send_str(b"\r\n");
            handle_input(&mut world, entity_id, &command_buffer);
        }
    }
}

fn handle_input(world: &mut World, entity_id: EntityId, command_buffer: &CommandBuffer) {
    let mut it: CommandBufferIterator = command_buffer.iter_words(|x| x.is_ascii_whitespace());
    match it.next() {
        Some(b"n") => action_go(world, entity_id, 0),
        Some(b"e") => action_go(world, entity_id, 1),
        Some(b"s") => action_go(world, entity_id, 2),
        Some(b"w") => action_go(world, entity_id, 3),
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
        _ => uart_send_str(b"not understood\r\n\r\n"),
    }
}

fn action_look(world: &World, entity_id: EntityId) {
    let entity = world.entities.get(entity_id).unwrap();
    let location = world.locations.get(entity.location).unwrap();
    uart_send_str(b"u r in ");
    uart_send_cstr(&location.name.data);

    uart_send_str(b"\r\nu c: ");
    let mut i = 0;
    for &oid in location.objects.iter() {
        if i != 0 {
            uart_send_str(b", ");
        }
        i += 1;
        uart_send_cstr(&world.objects.get(oid).unwrap().name.data);
    }
    if i == 0 {
        uart_send_str(b"nothing");
    }
    uart_send_str(b"\r\n");

    let mut i = 0;
    for &eid in location.entities.iter() {
        let e = world.entities.get(eid).unwrap();
        if eid != entity_id {
            if i != 0 {
                uart_send_str(b", ");
            }
            uart_send_cstr(&e.name.data);
            i += 1;
        }
    }
    if i > 0 {
        uart_send_str(b" is here\r\n");
    }

    uart_send_str(b"exits: ");
    let mut i = 0;
    for lid in location.links.iter() {
        if i != 0 {
            uart_send_str(b", ");
        }
        i += 1;
        uart_send_cstr(&world.links.get(lid.link).unwrap().name.data);
    }
    if i == 0 {
        uart_send_str(b"none");
    }
    uart_send_str(b"\r\n");
}

fn action_go(world: &mut World, entity_id: EntityId, link_id: LinkId) {
    let entity = world.entities.get_mut(entity_id).unwrap();
    let location = world.locations.get(entity.location).unwrap();

    // find "to" location id
    let to_location_id = match location.links.iter().find(|x| x.link == link_id) {
        Some(lnk) => lnk.location,
        None => {
            uart_send_str(b"can't go there\r\n\r\n");
            return;
        }
    };

    // add entity to new location
    world
        .locations
        .get_mut(to_location_id)
        .unwrap()
        .entities
        .push(entity_id);

    // remove entity from old location
    world
        .locations
        .get_mut(entity.location)
        .unwrap()
        .entities
        .retain(|&x| x != entity_id);

    // update entity location
    entity.location = to_location_id;
    uart_send_str(b"ok\r\n\r\n");
}

fn action_inventory(world: &World, entity_id: EntityId) {
    let entity = world.entities.get(entity_id).unwrap();
    uart_send_str(b"u have: ");
    let mut i = 0;
    for &oid in entity.objects.iter() {
        if i != 0 {
            uart_send_str(b", ");
        }
        i += 1;
        uart_send_cstr(&world.objects.get(oid).unwrap().name.data);
    }
    if i == 0 {
        uart_send_str(b"nothing");
    }
    uart_send_str(b"\r\n\r\n");
}

fn action_take(world: &mut World, entity_id: EntityId, it: &mut CommandBufferIterator) {
    let entity = world.entities.get_mut(entity_id).unwrap();
    let location = world.locations.get_mut(entity.location).unwrap();

    // get object name
    let object_name = match it.next() {
        Some(name) => name,
        None => {
            uart_send_str(b"take what?\r\n\r\n");
            return;
        }
    };

    // find object id and index in list
    let mut object_index = None;
    let mut object_id = None;

    for (index, &oid) in location.objects.iter().enumerate() {
        if world.objects.get(oid).unwrap().name.equals(object_name) {
            object_index = Some(index);
            object_id = Some(oid);
            break;
        }
    }

    let (object_index, object_id) = match object_id {
        Some(id) => (object_index.unwrap(), id),
        None => {
            uart_send_str(object_name);
            uart_send_str(b" is not here\r\n\r\n");
            return;
        }
    };

    // remove object from location
    location.objects.remove(object_index);

    // add object to entity
    entity.objects.push(object_id);

    uart_send_str(b"ok\r\n\r\n");
}

fn action_drop(world: &mut World, entity_id: EntityId, it: &mut CommandBufferIterator) {
    let entity = world.entities.get_mut(entity_id).unwrap();
    let object_name = match it.next() {
        Some(name) => name,
        None => {
            uart_send_str(b"drop what?\r\n\r\n");
            return;
        }
    };

    // find object id and index in entity inventory
    let mut object_index = None;
    let mut object_id = None;

    for (index, &oid) in entity.objects.iter().enumerate() {
        if world.objects.get(oid).unwrap().name.equals(object_name) {
            object_index = Some(index);
            object_id = Some(oid);
            break;
        }
    }

    let (object_index, object_id) = match object_id {
        Some(id) => (object_index.unwrap(), id),
        None => {
            uart_send_str(b"don't have ");
            uart_send_str(object_name);
            uart_send_str(b"\r\n\r\n");
            return;
        }
    };

    // remove object from entity
    entity.objects.remove(object_index);

    // add object to location
    world
        .locations
        .get_mut(entity.location)
        .unwrap()
        .objects
        .push(object_id);

    uart_send_str(b"ok\r\n\r\n");
}

fn action_give(world: &mut World, entity_id: EntityId, it: &mut CommandBufferIterator) {
    // get object name
    let object_name = match it.next() {
        Some(name) => name,
        None => {
            uart_send_str(b"give what?\r\n\r\n");
            return;
        }
    };

    // get entity name
    let to_entity_name = match it.next() {
        Some(name) => name,
        None => {
            uart_send_str(b"give to whom?\r\n\r\n");
            return;
        }
    };

    // find "to" entity
    let &to_entity_id = match world
        .locations
        .get(world.entities.get(entity_id).unwrap().location)
        .unwrap()
        .entities
        .iter()
        .find(|&&x| world.entities.get(x).unwrap().name.equals(to_entity_name))
    {
        Some(id) => id,
        None => {
            uart_send_str(to_entity_name);
            uart_send_str(b" not here\r\n\r\n");
            return;
        }
    };

    let from_entity = world.entities.get_mut(entity_id).unwrap();

    // find object in "from" entity
    let mut object_index = None;
    let mut object_id = None;

    for (index, &oid) in from_entity.objects.iter().enumerate() {
        if world.objects.get(oid).unwrap().name.equals(object_name) {
            object_index = Some(index);
            object_id = Some(oid);
            break;
        }
    }

    let (object_index, object_id) = match object_id {
        Some(id) => (object_index.unwrap(), id),
        None => {
            uart_send_str(b"don't have ");
            uart_send_str(object_name);
            uart_send_str(b"\r\n\r\n");
            return;
        }
    };

    // remove object from entity
    from_entity.objects.remove(object_index);

    // add object to "to" entity
    world
        .entities
        .get_mut(to_entity_id)
        .unwrap()
        .objects
        .push(object_id);

    uart_send_str(b"ok\r\n\r\n");
}

fn action_memory_info() {
    uart_send_str(b"   heap start: ");
    uart_send_hex_u32(memory_heap_start(), true);
    uart_send_str(b"\r\n");
    uart_send_str(b"heap position: ");
    uart_send_hex_u32(allocator_current_next() as u32, true);
    uart_send_str(b"\r\n");
    uart_send_str(b"stack pointer: ");
    uart_send_hex_u32(memory_stack_pointer(), true);
    uart_send_str(b"\r\n");
    uart_send_str(b"   memory end: ");
    uart_send_hex_u32(memory_end(), true);
    uart_send_str(b"\r\n\r\n");
}

fn action_sdcard_status() {
    uart_send_str(b"SDCARD_STATUS: 0x");
    uart_send_hex_u32(sdcard_status() as u32, true);
    uart_send_str(b"\r\n\r\n");
}

fn action_sdcard_read(it: &mut CommandBufferIterator) {
    let sector = match it.next() {
        Some(sector) => u8_slice_to_u32(sector),
        None => {
            uart_send_str(b"what sector\r\n\r\n");
            return;
        }
    };

    let mut buf = [0; 512];
    sdcard_read_blocking(sector, &mut buf);
    buf.iter().for_each(|&x| uart_send_char(x));
    uart_send_str(b"\r\n\r\n");
}

fn action_sdcard_write(it: &mut CommandBufferIterator) {
    let sector = match it.next() {
        Some(sector) => u8_slice_to_u32(sector),
        None => {
            uart_send_str(b"what sector\r\n\r\n");
            return;
        }
    };

    let rest = it.rest();
    let len = rest.len().min(512);
    let mut buf = [0u8; 512];
    buf[..len].copy_from_slice(&rest[..len]);
    sdcard_write_blocking(sector, &buf);
    uart_send_str(b"ok\r\n\r\n");
}

fn action_led_set(it: &mut CommandBufferIterator) {
    let bits = match it.next() {
        Some(bits) => u8_slice_to_u32(bits),
        None => {
            uart_send_str(b"which leds in bits 0 being on\r\n\r\n");
            return;
        }
    };

    led_set(bits as u8);
}

fn action_help() {
    uart_send_str(HELP);
}

fn action_new_object(world: &mut World, entity_id: EntityId, it: &mut CommandBufferIterator) {
    // get object name
    let object_name = match it.next() {
        Some(name) => name,
        None => {
            uart_send_str(b"what object name\r\n\r\n");
            return;
        }
    };

    let object = Object {
        name: Name::from(object_name),
    };

    let object_id = world.objects.len();
    world.objects.push(object);

    let entity = world.entities.get_mut(entity_id).unwrap();
    entity.objects.push(object_id);

    uart_send_str(b"ok\r\n\r\n");
}

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
        let ch = uart_read_char();
        led_set(!ch);

        match state {
            InputState::Normal => {
                if ch == CHAR_ESCAPE {
                    state = InputState::Escape;
                } else if ch == CHAR_BACKSPACE {
                    if command_buffer.backspace() {
                        uart_send_char(ch);
                        command_buffer.for_each_from_cursor(|c| uart_send_char(c));
                        uart_send_char(b' ');
                        uart_send_move_back(command_buffer.elements_after_cursor_count() + 1);
                    }
                } else if ch == CHAR_CARRIAGE_RETURN || command_buffer.is_full() {
                    return;
                } else {
                    if command_buffer.insert(ch) {
                        uart_send_char(ch);
                        command_buffer.for_each_from_cursor(|x| uart_send_char(x));
                        uart_send_move_back(command_buffer.elements_after_cursor_count());
                    }
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
                if ch >= b'0' && ch <= b'9' {
                    escape_sequence_parameter = escape_sequence_parameter * 10 + (ch - b'0');
                } else {
                    match ch {
                        b'D' => {
                            // arrow left
                            if command_buffer.move_cursor_left() {
                                uart_send_str(b"\x1B[D");
                            }
                        }
                        b'C' => {
                            // arrow right
                            if command_buffer.move_cursor_right() {
                                uart_send_str(b"\x1B[C");
                            }
                        }
                        b'~' => {
                            // delete
                            if escape_sequence_parameter == 3 {
                                // delete key
                                command_buffer.del();
                                command_buffer.for_each_from_cursor(|x| uart_send_char(x));
                                uart_send_char(b' ');
                                uart_send_move_back(
                                    command_buffer.elements_after_cursor_count() + 1,
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

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart_send_str(b"PANIC!!!");
    loop {}
}
