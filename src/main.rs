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
mod actions;
mod model;

extern crate alloc;

use actions::{
    ActionFailed, Result, action_drop, action_give, action_go, action_go_named_link, action_help,
    action_inventory, action_led_set, action_look, action_memory_info, action_new_entity,
    action_new_location, action_new_object, action_say, action_sdcard_read, action_sdcard_status,
    action_sdcard_write, action_set_location_note, action_take, action_tell, action_wait,
};
use alloc::vec;
use core::arch::global_asm;
use core::panic::PanicInfo;
use lib::api::{memory_end, uart_send_bytes, uart_send_move_back};
use lib::api_unsafe::{led_set, uart_read_byte, uart_send_byte};
use lib::cursor_buffer::{CursorBuffer, CursorBufferIterator};
use lib::global_allocator::GlobalAllocator;
use model::{Entity, EntityId, Location, Name, Note, World};

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
/// Will panic if `action_look` fails.
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
    separator_after_success: bool,
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

    if separator_after_success {
        uart_send_bytes(b"\r\n");
    }

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
