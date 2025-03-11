use quallocator::bump::{allocator::BumpAllocator, utils::get_current_heap};

fn main() {
    println!("Heap address: {:p}", get_current_heap());

    let word =
        BumpAllocator::qualloc::<char>((size_of::<char>() as i32) * 13).unwrap() as *mut [char; 13];

    // unsafe {
    //     *word = "Hello World!\n";
    // }
    // println!("Word address: {:p}", word);
    // unsafe {
    //     println!("Word value: {}", *word);
    // }
    println!("Heap address: {:p}", get_current_heap());
    BumpAllocator::qudelloc(word as *const ());

    println!("New heap address: {:p}", get_current_heap());
}
