use quallocator::bump::{qualloc::qualloc, qudelloc::qudelloc, utils::get_current_heap};

fn main() {
    println!("Heap address: {:p}", get_current_heap());
    let word = qualloc((size_of::<char>() as i32) * 13).unwrap() as *mut &str;

    unsafe {
        *word = "Hello World!\n";
    }
    println!("Word address: {:p}", word);
    unsafe {
        println!("Word value: {}", *word);
    }
    println!("Heap address: {:p}", get_current_heap());
    qudelloc(word as *const ());

    println!("New heap address: {:p}", get_current_heap());
}
