#![no_std]
#![no_main]

use core::arch::asm;
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

fn uart_send_hex_u32(i: u32, separate_half_words: bool) {
    uart_send_hex_byte((i >> 24) as u8);
    uart_send_hex_byte((i >> 16) as u8);
    if separate_half_words {
        uart_send_char(b':');
    }
    uart_send_hex_byte((i >> 8) as u8);
    uart_send_hex_byte(i as u8);
}

fn uart_send_hex_byte(ch: u8) {
    uart_send_hex_nibble(ch >> 4);
    uart_send_hex_nibble(ch & 0x0f);
}

fn uart_send_hex_nibble(nibble: u8) {
    if nibble < 10 {
        uart_send_char(b'0' + nibble);
    } else {
        uart_send_char(b'A' + (nibble - 10));
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
#[derive(Copy, Clone, PartialEq)]
struct Object {
    name: Name,
}

// Define the entity struct
#[derive(Copy, Clone, PartialEq)]
struct Entity {
    name: Name,
    location: LocationId,
    objects: FixedSizeList<ObjectId, MAX_OBJECTS_PER_ENTITY>,
}

// Define the entity struct
#[derive(Copy, Clone, PartialEq)]
struct Link {
    name: Name,
}

// Define the location_link struct
#[derive(Copy, Clone, PartialEq)]
struct LocationLink {
    link: LinkId,
    location: LocationId,
}

// Define the location struct
#[derive(Copy, Clone, PartialEq)]
struct Location {
    name: Name,
    links: FixedSizeList<LocationLink, MAX_LINKS_PER_LOCATION>,
    objects: FixedSizeList<ObjectId, MAX_OBJECTS_PER_LOCATION>,
    entities: FixedSizeList<EntityId, MAX_ENTITIES_PER_LOCATION>,
}

struct World {
    objects: FixedSizeList<Object, MAX_OBJECTS>,
    entities: FixedSizeList<Entity, MAX_ENTITIES>,
    locations: FixedSizeList<Location, MAX_LOCATIONS>,
    links: FixedSizeList<Link, MAX_LINKS>,
}

// Define the FixedSizeList struct
#[derive(Copy, Clone, PartialEq)]
struct FixedSizeList<T, const N: usize> {
    data: [Option<T>; N],
    count: usize,
}

impl<T: Copy + PartialEq, const N: usize> FixedSizeList<T, N> {
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

    fn remove(&mut self, item: T) -> bool {
        for i in 0..self.count {
            if self.data[i] == Some(item) {
                return self.remove_at(i);
            }
        }
        false
    }

    fn remove_at(&mut self, index: usize) -> bool {
        if index < self.count {
            self.data[index] = None;
            for i in index..self.count - 1 {
                self.data[i] = self.data[i + 1];
            }
            self.count -= 1;
            self.data[self.count] = None;
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

// struct Heap {
//     free: *mut u8,
// }

// static mut HEAP: Heap = Heap {
//     free: unsafe { &__heap_start__ as *const u8 as *mut u8 },
// };

// setup stack and jump to 'run()'
global_asm!(include_str!("startup.s"));

#[unsafe(no_mangle)]
pub extern "C" fn run() -> ! {
    let mut world = World {
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

    let mut entity_id = 1;
    loop {
        print_location(&world, entity_id);
        uart_send_str(world.entities.get(entity_id).unwrap().name);
        uart_send_str(b" > ");
        let mut cmdbuf = CommandBuffer::new();
        input(&mut cmdbuf);
        uart_send_str(b"\r\n");
        process_command(&mut world, entity_id, &cmdbuf);
        if entity_id == 1 {
            entity_id = 2;
        } else {
            entity_id = 1;
        }
    }
}

// fn malloc(size: usize) -> *mut u8 {
//     unsafe {
//         let ret = HEAP.free;
//         HEAP.free = ret.add(size);
//         ret
//     }
// }

fn process_command(world: &mut World, entity_id: EntityId, cmdbuf: &CommandBuffer) {
    let mut it = cmdbuf.iter_words();
    match it.next() {
        Some(b"n") => action_go(world, entity_id, 1),
        Some(b"e") => action_go(world, entity_id, 2),
        Some(b"s") => action_go(world, entity_id, 3),
        Some(b"w") => action_go(world, entity_id, 4),
        Some(b"i") => action_inventory(world, entity_id),
        Some(b"t") => action_take(world, entity_id, &mut it),
        Some(b"d") => action_drop(world, entity_id, &mut it),
        Some(b"g") => action_give(world, entity_id, &mut it),
        Some(b"sp") => action_stack_pointer(),
        _ => uart_send_str(b"not understood\r\n\r\n"),
    }

    // let cmd_len = cmd.len();
    // let name: &[u8];
    // unsafe {
    //     let mem = malloc(cmd_len);
    //     core::ptr::copy_nonoverlapping(cmd.as_ptr(), mem, cmd_len);
    //     name = core::slice::from_raw_parts(mem, cmd_len);
    // }
    // state.objects.add(Object { name: name });
    // state
    //     .locations
    //     .get_mut(1)
    //     .unwrap()
    //     .objects
    //     .add(state.objects.count - 1);
}

fn action_go(world: &mut World, entity_id: EntityId, link_id: LinkId) {
    let entity = world.entities.get_mut(entity_id).unwrap();
    let new_loc_id = {
        let loc = world.locations.get(entity.location).unwrap();
        let mut new_loc_id = 0;
        for ll in loc.links.iter() {
            if ll.link == link_id {
                new_loc_id = ll.location;
                break;
            }
        }
        new_loc_id
    };
    if new_loc_id == 0 {
        uart_send_str(b"can't go there\r\n\r\n");
        return;
    }

    // add entity to new location
    if !world
        .locations
        .get_mut(new_loc_id)
        .unwrap()
        .entities
        .add(entity_id)
    {
        return;
    }
    // remove entity from old location
    if !world
        .locations
        .get_mut(entity.location)
        .unwrap()
        .entities
        .remove(entity_id)
    {
        return;
    }
    // update entity location
    entity.location = new_loc_id;
}

fn action_inventory(world: &World, entity_id: EntityId) {
    let entity = world.entities.get(entity_id).unwrap();
    uart_send_str(b"u have: ");
    let mut i = 0;
    for oid in entity.objects.iter() {
        if i != 0 {
            uart_send_str(b", ");
        }
        i += 1;
        uart_send_str(world.objects.get(oid).unwrap().name);
    }
    if i == 0 {
        uart_send_str(b"nothing");
    }
    uart_send_str(b"\r\n\r\n");
}

fn action_take(world: &mut World, entity_id: EntityId, it: &mut CommandBufferIterator) {
    let entity = world.entities.get_mut(entity_id).unwrap();
    let loc = world.locations.get_mut(entity.location).unwrap();
    let object_name = it.next();
    if object_name.is_none() {
        uart_send_str(b"take what?\r\n\r\n");
        return;
    }
    let object_name = object_name.unwrap();
    for oid in loc.objects.iter() {
        let obj = world.objects.get(oid).unwrap();
        if obj.name != object_name {
            continue;
        }
        // remove object from location
        if !loc.objects.remove(oid) {
            uart_send_str(b"error\r\n\r\n");
            return;
        }
        // add object to entity
        if !entity.objects.add(oid) {
            uart_send_str(b"error\r\n\r\n");
            return;
        }
        uart_send_str(b"ok\r\n\r\n");
        return;
    }
    uart_send_str(object_name);
    uart_send_str(b" is not here\r\n\r\n");
}

fn action_drop(world: &mut World, entity_id: EntityId, it: &mut CommandBufferIterator) {
    let entity = world.entities.get_mut(entity_id).unwrap();
    let object_name = it.next();
    if object_name.is_none() {
        uart_send_str(b"drop what?\r\n\r\n");
        return;
    }
    let object_name = object_name.unwrap();
    for oid in entity.objects.iter() {
        let obj = world.objects.get(oid).unwrap();
        if obj.name != object_name {
            continue;
        }
        // remove object from entity
        if !entity.objects.remove(oid) {
            uart_send_str(b"error\r\n\r\n");
            return;
        }
        // add object to location
        if !world
            .locations
            .get_mut(entity.location)
            .unwrap()
            .objects
            .add(oid)
        {
            uart_send_str(b"error\r\n\r\n");
            return;
        }
        uart_send_str(b"ok\r\n\r\n");
        return;
    }
    uart_send_str(b"don't have ");
    uart_send_str(object_name);
    uart_send_str(b"\r\n\r\n");
}

fn action_give(world: &mut World, entity_id: EntityId, it: &mut CommandBufferIterator) {
    let object_name = it.next();
    if object_name.is_none() {
        uart_send_str(b"give what?\r\n\r\n");
        return;
    }
    let object_name = object_name.unwrap();
    let to_entity_name = it.next();
    if to_entity_name.is_none() {
        uart_send_str(b"give to who?\r\n\r\n");
        return;
    }
    let to_entity_name = to_entity_name.unwrap();
    let mut to_entity_id = 0;
    for i in world
        .locations
        .get(world.entities.get(entity_id).unwrap().location)
        .unwrap()
        .entities
        .iter()
    {
        let e = world.entities.get(i).unwrap();
        if e.name == to_entity_name {
            to_entity_id = i;
            break;
        }
    }
    if to_entity_id == 0 {
        uart_send_str(to_entity_name);
        uart_send_str(b" not here\r\n\r\n");
        return;
    }
    let entity = world.entities.get_mut(entity_id).unwrap();
    for oid in entity.objects.iter() {
        let obj = world.objects.get(oid).unwrap();
        if obj.name != object_name {
            continue;
        }
        // remove object from entity
        if !entity.objects.remove(oid) {
            uart_send_str(b"error\r\n\r\n");
            return;
        }
        // add object to location
        if !world
            .entities
            .get_mut(to_entity_id)
            .unwrap()
            .objects
            .add(oid)
        {
            uart_send_str(b"error\r\n\r\n");
            return;
        }
        uart_send_str(b"ok\r\n\r\n");
        return;
    }
    uart_send_str(b"don't have ");
    uart_send_str(object_name);
    uart_send_str(b"\r\n\r\n");
}

fn action_stack_pointer() {
    uart_send_str(b"stack pointer: ");
    let sp: u32;
    unsafe {
        asm!(
            "mv {0}, sp", // Move the stack pointer to the output variable
            out(reg) sp,
        );
    }
    uart_send_hex_u32(sp, true);
    uart_send_str(b"\r\n\r\n");
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

fn print_location(world: &World, entity_id: EntityId) {
    let entity = world.entities.get(entity_id).unwrap();
    let loc = world.locations.get(entity.location).unwrap();
    uart_send_str(b"u r in ");
    uart_send_str(loc.name);

    uart_send_str(b"\r\nu c: ");
    let mut i = 0;
    for oid in loc.objects.iter() {
        if i != 0 {
            uart_send_str(b", ");
        }
        i += 1;
        uart_send_str(world.objects.get(oid).unwrap().name);
    }
    if i == 0 {
        uart_send_str(b"nothing");
    }
    uart_send_str(b"\r\n");

    let mut i = 0;
    for eid in loc.entities.iter() {
        let e = world.entities.get(eid).unwrap();
        if e != entity {
            if i != 0 {
                uart_send_str(b", ");
            }
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
        uart_send_str(world.links.get(lid.link).unwrap().name);
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
