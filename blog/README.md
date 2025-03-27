# My first days with Rust from the perspective of an experienced C++ programmer

My main focus is bare metal applications. No standard libraries and building RISC-V RV32I binary running on a FPGA implementation.

### Day 0
Got bare metal binary running echo application on the FPGA emulator. Surprisingly easy doing low level hardware interactions in unsafe mode. Back and forth with multiple AI's with questions such as: How would this be written in Rust considering this C++ code?

## Day 1
Implementing toy test application from C++ to Rust dabbling with data structure using references. Ultimately defeated and settling for "index in vectors" based data structures.

Is there other way except Rc<RefCell<...>> considering the borrow checker.

## Day 2
Got toy application working on FPGA with peripherals. Total success and pleased with the result of 3 days Rust from scratch!

Next is reading the rust-book and maybe some references on what is available in no_std mode

## Day 3
Using AIs with questions such as how do I do this and that in Rust describing things that I know are there makes the transition smooth.

What first seemed like elaborate syntax makes perfect sense and probably as good as it can be.

I will read the Rust book and the reference to get formally educated but for now AI acts as a tutor answering things that it has seen plenty of times, noob questions.

The binary is larger, as expected, primarily (I think) due to the initial data structure is built in a function instead of hard-coded as a global.

Somewhat larger binary is expected and acceptable due to the built in safeties of Rust.

Without AI the learning curve is a bit steep and for a programming noob is probably off-putting. For an experienced C++ programmer is just: "yeah, that's better" and it keeps giving me a tiny smile every time that happens.

I begin to understand the cult like following Rust has because once a learning step in the curve is taken it feels like there is no going back.

I have a lot to learn, but for now, for my toy bare-metal application, I feel that this is the way forward.

p.s. I was pleasantly surprised by how extensive the core library is and that it works in [no_std] builds.

## Day 4. To the heap
Getting the hang of data on the stack: done. It is now time to move to the heap.

The simplest bump allocator implemented and Rust can now allocate memory. Figured out how to / if use Box to allocate on the heap.

Pleased to notice that an object type has been "unlocked": Vec.

The fixed sized list has been retired and now experimenting with heap allocations.

Started by placing names of objects on the heap with Box but settled for fixed size array in the struct for better cache coherence. Then moved the name to a struct and with a basic impl improved the ergonomics of comparing and initiating names.

So far everything is moving along smoothly.

AIs are fantastic at tutoring the noob questions.

With a background in C++ everything so far makes sense. However, for a programming noob, it is just to much to know at once before being able to do something meaningful.

Looking forward to acquire the formal knowledge from the Rust book and reference.

## Day 5. Strings
Pondering whether to use String or custom data type for "name". Fixed size `u8` array gives better cache coherence although it does not matter in this application. String gives built-in UTF-8 support but results in more allocations.

Log of allocator output for the `[u8;NAME_SIZE]` custom data type and using String.

Custom data type using `[u8;NAME_SIZE]` gives:
```
binary size: 14000 B
alloc: at 0000:36C0 size: 0000:0080
alloc: at 0000:3740 size: 0000:0010
alloc: at 0000:3750 size: 0000:00C0
alloc: at 0000:3810 size: 0000:0020
alloc: at 0000:3830 size: 0000:0010
alloc: at 0000:3840 size: 0000:0110
alloc: at 0000:3950 size: 0000:0020
alloc: at 0000:3970 size: 0000:0010
alloc: at 0000:3980 size: 0000:0010
alloc: at 0000:3990 size: 0000:0020
alloc: at 0000:39B0 size: 0000:0080
alloc: at 0000:3A30 size: 0000:0100
de-alloc: at 0000:39B0 size: 0000:0080
```

Using String gives:
```
binary size: 16400 B
alloc: at 0000:4020 size: 0000:0008
alloc: at 0000:4028 size: 0000:0030
alloc: at 0000:4058 size: 0000:0006
alloc: at 0000:405E size: 0000:0007
alloc: at 0000:4065 size: 0000:0002
alloc: at 0000:4068 size: 0000:0010
alloc: at 0000:4078 size: 0000:0070
alloc: at 0000:40E8 size: 0000:0001
alloc: at 0000:40E9 size: 0000:0005
alloc: at 0000:40F0 size: 0000:0020
alloc: at 0000:4110 size: 0000:0010
alloc: at 0000:4120 size: 0000:00C0
alloc: at 0000:41E0 size: 0000:0006
alloc: at 0000:41E8 size: 0000:0020
alloc: at 0000:4208 size: 0000:0010
alloc: at 0000:4218 size: 0000:0010
alloc: at 0000:4228 size: 0000:0008
alloc: at 0000:4230 size: 0000:0007
alloc: at 0000:4238 size: 0000:0020
alloc: at 0000:4258 size: 0000:0005
alloc: at 0000:4260 size: 0000:0030
alloc: at 0000:4290 size: 0000:0004
alloc: at 0000:4294 size: 0000:0005
alloc: at 0000:4299 size: 0000:0004
alloc: at 0000:429D size: 0000:0002
alloc: at 0000:42A0 size: 0000:0060
de-alloc: at 0000:4260 size: 0000:0030
```
## Day 6. roome
Refining the use of built-in operations in iter. Powerful functions such as find, position, find_map, enumeration and any explored.

Getting acquainted with foundational trait Default and implementing it for some structs.

Borrow checker will not allow passing a mutable reference to a function as immutable although single threaded, thus safe.

... unless it is an argument to called function and not borrowed prior to the target call.

Settling for no UTF-8 support and no String use yet.

Using AIs to examine code and suggest changes where the Rust idioms are not used.

The creation script is done and roome is created.

## Day 7. Formal
Started reading "Programming Rust" by Jim Blandy amongst others for a summary formal overview before exploring reference manuals and reading the rust-book.