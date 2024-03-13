#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box,rc::Rc};
use blog_os::println;
use core::panic::PanicInfo;
use bootloader::{entry_point, BootInfo};
use blog_os::task::{Task, simple_executor::SimpleExecutor};
use blog_os::task::keyboard;
use blog_os::task::executor::Executor;

entry_point!(kernel_main);


fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use x86_64::{VirtAddr, structures::paging::{Page, Size4KiB}};
    use blog_os::memory::{self, BootInfoFrameAllocator};
    use blog_os::allocator;
    use alloc::vec; // Add this line to import the `vec` macro

    let _page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(0xdeadbeef000));

    println!("Hello World{}", "!");
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialisation failed");

    // allocate a number on the heap
    let heap_value = Box::new(41);
    println!("heap value at {:p}", heap_value);

    // create a dynamically sized vector
    let mut vec = vec![]; // Replace `Vec::new()` with `vec![]`

    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reached 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    // map an unused page
    let page = Page::containing_address(VirtAddr::new(0));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};

    let mut executor = Executor::new();
    executor::spawn(Task::new(example_task()));
    executor.run();

    #[cfg(test)]
    test_main();

    let mut executor = SimpleExecutor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    println!("It did not crash!");
    blog_os::hlt_loop();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    blog_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}