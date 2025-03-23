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
