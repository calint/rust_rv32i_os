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

static HELP: &[u8] = b"command:\r
\x20 go <exit>: go\r
\x20 n: go north\r
\x20 e: go east\r
\x20 s: go south\r
\x20 w: go west\r
\x20 i: display inventory\r
\x20 t <object>: take object\r
\x20 d <object>: drop object\r
\x20 g <entity> <object>: give entity object from inventory\r
\x20 say <what>: say to all in location\r
\x20 tell <whom> <what>: tells entity in location\r
\x20 sln <text>: set location note\r
\x20 sds: SD card status (6 is ok)\r
\x20 sdr <sector>: read sector from SD card\r
\x20 sdw <sector> <text>: write sector to SD card\r
\x20 led <bits with 1 being on>: turn on/off leds\r
\x20 no <object name>: new object into current inventory\r
\x20 nl <to link> <back link> <new location name>: new linked location\r
\x20 ne <name>: create new entity\r
\x20 mi: memory allocation info\r
\x20 wait: do nothing\r
\x20 help: this message\r\n";

static CREATION: &[u8] = b"sln todo: find an exit
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

extern crate alloc;

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

use actions::{
    ActionContext, ActionError, CommandBuffer, Result, action_drop, action_give, action_go,
    action_go_named_link, action_help, action_inventory, action_led_set, action_look,
    action_memory_info, action_new_entity, action_new_location, action_new_object, action_say,
    action_sdcard_read, action_sdcard_status, action_sdcard_write, action_set_location_note,
    action_take, action_tell, action_wait,
};
use alloc::vec;
use core::arch::global_asm;
use core::panic::PanicInfo;
use lib::api::{Printer, PrinterUART, PrinterVoid, memory_end};
use lib::api_unsafe::{led_set, uart_read_byte};
use lib::global_allocator::GlobalAllocator;
use model::{Entity, Location, Name, Note, World};

const CHAR_CARRIAGE_RETURN: u8 = 0xd;
const CHAR_BACKSPACE: u8 = 0x7f;
const CHAR_ESCAPE: u8 = 0x1b;
const CHAR_CTRL_A: u8 = 1;
const CHAR_CTRL_E: u8 = 5;
const CHAR_FORM_FEED: u8 = 0xc;
const CHAR_MOVE_CURSOR_BACK: u8 = 8;

// Setup bss section, stack and jump to `run()`.
global_asm!(include_str!("startup.s"));

/// # Panics
///
/// Will panic if `action_look` fails.
#[unsafe(no_mangle)]
pub extern "C" fn run() -> ! {
    led_set(0b0000); // turn all leds on

    GlobalAllocator::init(memory_end() as usize);

    let mut world = create_world();

    let mut printer = PrinterUART::new();

    printer.p(ASCII_ART);
    printer.p(HELLO);

    loop {
        for entity_id in 0..world.entities.len() {
            {
                // note: for consistency `action_look` requires `ActionContext`
                let command_buffer = CommandBuffer::new();
                let mut ctx = ActionContext {
                    printer: &mut printer,
                    world: &mut world,
                    entity_id,
                    tokens: &mut command_buffer.iter_tokens(u8::is_ascii_whitespace),
                };

                assert!(action_look(&mut ctx).is_ok(), "cannot look");
            }

            loop {
                // loop until action succeeded

                printer.p(&world.entities[entity_id].name);
                printer.p(b" > ");

                let mut command_buffer = CommandBuffer::new();
                input(&mut command_buffer, &printer);
                printer.nl();

                let mut ctx = ActionContext {
                    printer: &mut printer,
                    world: &mut world,
                    entity_id,
                    tokens: &mut command_buffer.iter_tokens(u8::is_ascii_whitespace),
                };

                if handle_input(&mut ctx).is_ok() {
                    break;
                }
            }
        }
    }
}

fn handle_input(ctx: &mut ActionContext) -> Result<()> {
    match ctx.tokens.next() {
        Some(b"go") => action_go(ctx)?,
        Some(b"n") => action_go_named_link(ctx, b"north")?,
        Some(b"e") => action_go_named_link(ctx, b"east")?,
        Some(b"s") => action_go_named_link(ctx, b"south")?,
        Some(b"w") => action_go_named_link(ctx, b"west")?,
        Some(b"i") => action_inventory(ctx)?,
        Some(b"t") => action_take(ctx)?,
        Some(b"d") => action_drop(ctx)?,
        Some(b"g") => action_give(ctx)?,
        Some(b"say") => action_say(ctx)?,
        Some(b"tell") => action_tell(ctx)?,
        Some(b"sln") => action_set_location_note(ctx)?,
        Some(b"sds") => action_sdcard_status(ctx)?,
        Some(b"sdr") => action_sdcard_read(ctx)?,
        Some(b"sdw") => action_sdcard_write(ctx)?,
        Some(b"led") => action_led_set(ctx)?,
        Some(b"no") => action_new_object(ctx)?,
        Some(b"nl") => action_new_location(ctx)?,
        Some(b"ne") => action_new_entity(ctx)?,
        Some(b"mi") => action_memory_info(ctx)?,
        Some(b"wait") => action_wait(ctx)?,
        Some(b"help") => action_help(ctx, HELP)?,
        _ => {
            ctx.printer.p(b"not understood");
            ctx.printer.nlc(2);
            return Err(ActionError::NotUnderstood);
        }
    }

    ctx.printer.nl();

    Ok(())
}

fn input(command_buffer: &mut CommandBuffer, printer: &PrinterUART) {
    loop {
        let ch = uart_read_byte();
        led_set(!ch);

        match ch {
            CHAR_ESCAPE => input_escape_sequence(command_buffer, printer),
            CHAR_BACKSPACE => input_backspace(command_buffer, printer),
            CHAR_CARRIAGE_RETURN => return,
            CHAR_FORM_FEED => {} // ignore CTRL+L
            CHAR_CTRL_A => input_move_to_start_of_line(command_buffer, printer),
            CHAR_CTRL_E => input_move_to_end_of_line(command_buffer, printer),
            _ if command_buffer.is_full() => return,
            _ => input_normal_char(command_buffer, printer, ch),
        }
    }
}

fn input_escape_sequence(command_buffer: &mut CommandBuffer, printer: &PrinterUART) {
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
                    if command_buffer.move_cursor_left().is_ok() {
                        printer.p(b"\x1B[D");
                    }
                }
                b'C' => {
                    if command_buffer.move_cursor_right().is_ok() {
                        printer.p(b"\x1B[C");
                    }
                }
                b'~' => {
                    if command_buffer.delete().is_ok() {
                        command_buffer.for_each_from_cursor(|&x| printer.pb(x));
                        printer.pb(b' ');
                        let count = command_buffer.elements_after_cursor_count() + 1;
                        // note: +1 because of ' ' that erases the trailing character
                        for _ in 0..count {
                            printer.pb(CHAR_MOVE_CURSOR_BACK);
                        }
                    }
                }
                _ => {}
            }
            return;
        }
    }
}

fn input_backspace(command_buffer: &mut CommandBuffer, printer: &PrinterUART) {
    if command_buffer.backspace().is_ok() {
        printer.pb(CHAR_BACKSPACE);
        command_buffer.for_each_from_cursor(|&x| printer.pb(x));
        printer.pb(b' ');
        let count = command_buffer.elements_after_cursor_count() + 1;
        // note: +1 because of ' ' that erases the trailing character
        for _ in 0..count {
            printer.pb(CHAR_MOVE_CURSOR_BACK);
        }
    }
}

fn input_normal_char(command_buffer: &mut CommandBuffer, printer: &PrinterUART, ch: u8) {
    if command_buffer.insert(ch).is_ok() {
        printer.pb(ch);
        command_buffer.for_each_from_cursor(|&x| printer.pb(x));
        let count = command_buffer.elements_after_cursor_count();
        for _ in 0..count {
            printer.pb(CHAR_MOVE_CURSOR_BACK);
        }
    }
}

fn input_move_to_start_of_line(command_buffer: &mut CommandBuffer, printer: &PrinterUART) {
    let steps = command_buffer.move_cursor_to_start_of_line();
    if steps != 0 {
        printer.p(b"\x1B[");
        if steps > 1 {
            printer.p_u32(u32::try_from(steps).expect("steps out of range"));
        }
        printer.p(b"D");
    }
}

fn input_move_to_end_of_line(command_buffer: &mut CommandBuffer, printer: &PrinterUART) {
    let steps = command_buffer.move_cursor_to_end_of_line();
    if steps != 0 {
        printer.p(b"\x1B[");
        if steps > 1 {
            printer.p_u32(u32::try_from(steps).expect("steps out of range"));
        }
        printer.p(b"C");
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
            assert!(command_buffer.insert(byte).is_ok(), "command too large");
        }

        let mut ctx = ActionContext {
            printer: &mut PrinterVoid::new(),
            world: &mut world,
            entity_id: 0,
            tokens: &mut command_buffer.iter_tokens(u8::is_ascii_whitespace),
        };

        assert!(handle_input(&mut ctx).is_ok(), "error creating world");

        // clear messages on all entities in case input generated messages
        world.entities.iter_mut().for_each(|x| x.messages.clear());
    }

    world
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    led_set(0b0000); // turn on all leds
    PrinterUART::new().pl(b"PANIC!!!");
    loop {}
}
