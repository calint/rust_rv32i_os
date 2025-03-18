#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

mod constants; // FPGA addresses
use constants::*;

unsafe extern "C" {
    // declared in 'linker.ld'
    static __heap_start__: u8;
}

// API
#[inline(always)]
fn uart_read_char() -> u8 {
    loop {
        unsafe {
            let input = read_volatile(UART_IN_ADDR as *const i32);
            if input == -1 {
                continue;
            }
            return input as u8;
        }
    }
}

#[inline(always)]
fn uart_send_char(ch: u8) {
    unsafe {
        while read_volatile(UART_OUT_ADDR as *const i32) != -1 {}
        write_volatile(UART_OUT_ADDR as *mut i32, ch as i32);
    }
}

// #[inline(always)]
// fn uart_send_cstr(cstr: *const u8) {
//     unsafe {
//         let mut ptr = cstr;
//         while *ptr != 0 {
//             while read_volatile(UART_OUT_ADDR as *const i32) != -1 {}
//             write_volatile(UART_OUT_ADDR as *mut i32, *ptr as i32);
//             ptr = ptr.offset(1);
//         }
//     }
// }

#[inline(always)]
fn uart_send_str(str: &[u8]) {
    for &byte in str {
        uart_send_char(byte);
    }
}

const MAX_OBJECTS: usize = 128;
const MAX_ENTITIES: usize = 128;
const MAX_LOCATIONS: usize = 128;
const MAX_LINKS: usize = 128;
const MAX_LINKS_PER_LOCATION: usize = 128;
const MAX_OBJECTS_PER_LOCATION: usize = 128;
const MAX_ENTITIES_PER_LOCATION: usize = 128;
const MAX_OBJECTS_PER_ENTITY: usize = 128;

// Define type aliases
type Name = &'static [u8];
type LocationId = usize;
type LinkId = usize;
type EntityId = usize;
type ObjectId = usize;

// Define the object struct
#[derive(Debug, Copy, Clone)]
struct Object {
    name: Name,
}

// Define the entity struct
#[derive(Debug, Copy, Clone, PartialEq)]
struct Entity {
    name: Name,
    location: LocationId,
    objects: FixedSizeList<ObjectId, MAX_OBJECTS_PER_ENTITY>,
}

// Define the entity struct
#[derive(Debug, Copy, Clone)]
struct Link {
    name: Name,
}

// Define the location_link struct
#[derive(Debug, Copy, Clone)]
struct LocationLink {
    link: LinkId,
    location: LocationId,
}

// Define the location struct
#[derive(Debug, Copy, Clone)]
struct Location {
    name: Name,
    links: FixedSizeList<LocationLink, MAX_LINKS_PER_LOCATION>,
    objects: FixedSizeList<ObjectId, MAX_OBJECTS_PER_LOCATION>,
    entities: FixedSizeList<EntityId, MAX_ENTITIES_PER_LOCATION>,
}

struct State {
    objects: FixedSizeList<Object, MAX_OBJECTS>,
    entities: FixedSizeList<Entity, MAX_ENTITIES>,
    locations: FixedSizeList<Location, MAX_LOCATIONS>,
    links: FixedSizeList<Link, MAX_LINKS>,
}

// Define the FixedSizeList struct
#[derive(Debug, Copy, Clone, PartialEq)]
struct FixedSizeList<T, const N: usize> {
    data: [Option<T>; N],
    count: usize,
}

impl<T: Copy, const N: usize> FixedSizeList<T, N> {
    fn new() -> Self {
        FixedSizeList {
            data: [None; N],
            count: 0,
        }
    }

    fn add(&mut self, item: T) -> bool {
        if self.count < N {
            self.data[self.count] = Some(item);
            self.count += 1;
            true
        } else {
            false
        }
    }

    fn get(&self, index: usize) -> Option<&T> {
        if index < self.count {
            self.data[index].as_ref()
        } else {
            None
        }
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.count {
            self.data[index].as_mut()
        } else {
            None
        }
    }

    fn iter(&self) -> FixedSizeListIterator<T, N> {
        FixedSizeListIterator {
            list: self,
            index: 0,
        }
    }
}

struct FixedSizeListIterator<'a, T, const N: usize> {
    list: &'a FixedSizeList<T, N>,
    index: usize,
}

impl<'a, T: Copy, const N: usize> Iterator for FixedSizeListIterator<'a, T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.list.count {
            let item = self.list.data[self.index];
            self.index += 1;
            item
        } else {
            None
        }
    }
}

struct CommandBuffer {
    buffer: [u8; 80],
    count: usize,
}

impl CommandBuffer {
    fn new() -> Self {
        CommandBuffer {
            buffer: [0; 80],
            count: 0,
        }
    }

    fn insert(&mut self, ch: u8) -> bool {
        if self.count < 80 {
            self.buffer[self.count] = ch;
            self.count += 1;
            true
        } else {
            false
        }
    }

    fn backspace(&mut self) -> bool {
        if self.count > 0 {
            self.count -= 1;
            true
        } else {
            false
        }
    }

    // iterate over the buffer returning a slice for each word
    fn iter_words(&self) -> CommandBufferIterator {
        CommandBufferIterator {
            cmdbuf: self,
            index: 0,
        }
    }
}

// iterator over the command buffer returning a slice for each word
struct CommandBufferIterator<'a> {
    cmdbuf: &'a CommandBuffer,
    index: usize,
}

impl<'a> Iterator for CommandBufferIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.cmdbuf.count {
            let start = self.index;
            while self.index < self.cmdbuf.count
                && !self.cmdbuf.buffer[self.index].is_ascii_whitespace()
            {
                self.index += 1;
            }
            let end = self.index;
            while self.index < self.cmdbuf.count
                && self.cmdbuf.buffer[self.index].is_ascii_whitespace()
            {
                self.index += 1;
            }
            Some(&self.cmdbuf.buffer[start..end])
        } else {
            None
        }
    }
}

struct Heap {
    free: *mut u8,
}

static mut HEAP: Heap = Heap {
    free: unsafe { &__heap_start__ as *const u8 as *mut u8 },
};

// setup stack and jump to 'run()'
global_asm!(include_str!("startup.s"));

#[unsafe(no_mangle)]
pub extern "C" fn run() -> ! {
    let mut state = State {
        objects: {
            let mut objects = FixedSizeList::new();
            objects.add(Object { name: b"" });
            objects.add(Object { name: b"notebook" });
            objects.add(Object { name: b"mirror" });
            objects.add(Object { name: b"lighter" });
            objects
        },
        entities: {
            let mut entities = FixedSizeList::new();
            entities.add(Entity {
                name: b"",
                location: 0,
                objects: FixedSizeList::new(),
            });
            entities.add(Entity {
                name: b"me",
                location: 1,
                objects: {
                    let mut list = FixedSizeList::new();
                    list.add(2);
                    list
                },
            });
            entities.add(Entity {
                name: b"u",
                location: 2,
                objects: FixedSizeList::new(),
            });
            entities
        },
        locations: {
            let mut locations = FixedSizeList::new();
            locations.add(Location {
                name: b"",
                links: FixedSizeList::new(),
                objects: FixedSizeList::new(),
                entities: FixedSizeList::new(),
            });
            locations.add(Location {
                name: b"roome",
                links: {
                    let mut list = FixedSizeList::new();
                    list.add(LocationLink {
                        link: 1,
                        location: 2,
                    });
                    list.add(LocationLink {
                        link: 2,
                        location: 3,
                    });
                    list.add(LocationLink {
                        link: 4,
                        location: 4,
                    });
                    list
                },
                objects: FixedSizeList::new(),
                entities: {
                    let mut list = FixedSizeList::new();
                    list.add(1);
                    list
                },
            });
            locations.add(Location {
                name: b"office",
                links: {
                    let mut list = FixedSizeList::new();
                    list.add(LocationLink {
                        link: 3,
                        location: 1,
                    });
                    list
                },
                objects: {
                    let mut list = FixedSizeList::new();
                    list.add(1);
                    list.add(3);
                    list
                },
                entities: {
                    let mut list = FixedSizeList::new();
                    list.add(2);
                    list
                },
            });
            locations.add(Location {
                name: b"bathroom",
                links: FixedSizeList::new(),
                objects: FixedSizeList::new(),
                entities: FixedSizeList::new(),
            });
            locations.add(Location {
                name: b"kitchen",
                links: {
                    let mut list = FixedSizeList::new();
                    list.add(LocationLink {
                        link: 2,
                        location: 1,
                    });
                    list
                },
                objects: FixedSizeList::new(),
                entities: FixedSizeList::new(),
            });
            locations
        },
        links: {
            let mut links = FixedSizeList::new();
            links.add(Link { name: b"" });
            links.add(Link { name: b"north" });
            links.add(Link { name: b"east" });
            links.add(Link { name: b"south" });
            links.add(Link { name: b"west" });
            links.add(Link { name: b"up" });
            links.add(Link { name: b"down" });
            links
        },
    };

    loop {
        print_location(&state, 1);
        uart_send_str(state.entities.get(1).unwrap().name);
        uart_send_str(b" > ");
        let mut cmdbuf = CommandBuffer::new();
        input(&mut cmdbuf);
        uart_send_str(b"\r\n");

        process_command(&mut state, &cmdbuf);
    }
}

fn malloc(size: usize) -> *mut u8 {
    unsafe {
        let ret = HEAP.free;
        HEAP.free = ret.add(size);
        ret
    }
}

fn process_command(state: &mut State, cmdbuf: &CommandBuffer) {
    let mut it = cmdbuf.iter_words();
    if let Some(cmd) = it.next() {
        let cmd_len = cmd.len();
        let name: &[u8];
        unsafe {
            let mem = malloc(cmd_len);
            core::ptr::copy_nonoverlapping(cmd.as_ptr(), mem, cmd_len);
            name = core::slice::from_raw_parts(mem, cmd_len);
        }
        state.objects.add(Object { name: name });
        state
            .locations
            .get_mut(1)
            .unwrap()
            .objects
            .add(state.objects.count - 1);
    } else {
        uart_send_str(b"not understood\r\n");
    }
}

fn input(cmdbuf: &mut CommandBuffer) {
    loop {
        let ch = uart_read_char();
        if ch == b'\r' {
            break;
        }
        if ch == b'\n' {
            continue;
        }
        if ch == 0x7f {
            if cmdbuf.backspace() {
                uart_send_char(0x7f);
                uart_send_char(b' ');
                uart_send_char(0x7f);
            }
            continue;
        }
        if cmdbuf.insert(ch) {
            uart_send_char(ch);
        }
    }
}

fn print_location(state: &State, entity_id: EntityId) {
    let entity = state.entities.get(entity_id).unwrap();
    let loc = state.locations.get(entity.location).unwrap();
    uart_send_str(b"u r in ");
    uart_send_str(loc.name);

    uart_send_str(b"\r\nu c: ");
    let mut i = 0;
    for oid in loc.objects.iter() {
        if i != 0 {
            uart_send_str(b", ");
        }
        i += 1;
        uart_send_str(state.objects.get(oid).unwrap().name);
    }
    if i == 0 {
        uart_send_str(b"nothing");
    }
    uart_send_str(b"\r\n");

    let mut i = 0;
    for eid in loc.entities.iter() {
        if i != 0 {
            uart_send_str(b", ");
        }
        let e = state.entities.get(eid).unwrap();
        if e != entity {
            uart_send_str(e.name);
            i += 1;
        }
    }
    if i > 0 {
        uart_send_str(b" is here\r\n");
    }

    uart_send_str(b"exits: ");
    let mut i = 0;
    for lid in loc.links.iter() {
        if i != 0 {
            uart_send_str(b", ");
        }
        i += 1;
        uart_send_str(state.links.get(lid.link).unwrap().name);
    }
    if i == 0 {
        uart_send_str(b"none");
    }
    uart_send_str(b"\r\n");
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart_send_str(b"PANIC!!!");
    loop {}
}
