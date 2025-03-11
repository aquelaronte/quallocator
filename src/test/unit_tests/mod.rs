use crate::bump::{allocator::BumpAllocator, utils::{align_up, get_current_heap}, BumpMemoryBlockHeader};
use libc::sbrk;

#[test]
fn test_get_current_heap() {
    let heap_address = get_current_heap();
    unsafe {
        sbrk(32);
    }

    let new_heap_address = get_current_heap();
    assert_ne!(heap_address, new_heap_address);
    assert_eq!(heap_address as i32 + 32, new_heap_address as i32);

    #[cfg(not(target_os = "macos"))]
    {
        unsafe {
            sbrk(-32);
        }
        let reduced_heap_address = get_current_heap();
        assert_eq!(heap_address, reduced_heap_address);
        assert_eq!(new_heap_address as i32 - 32, reduced_heap_address as i32);
    }
}

#[test]
fn test_align_up() {
    /*
     * 13 is not a multiple of the char alignment, so it should be rounded up
     *
     * char alignment constant is 4 on x86_64, so 13 rounded up is 16
     */
    let aligned_size = align_up::<char>(13);
    let char_alignment = align_of::<char>() as i32;

    assert!(aligned_size % char_alignment == 0);

    /*
     * u8 alignment constant is 1 on x86_64, so every number is aligned
     * even is it's a prime number
     */
    let aligned_size = align_up::<u8>(17);
    assert_eq!(aligned_size, 17);
}

#[test]
fn test_qualloc() {
    let heap_address = get_current_heap();
    let word_size = size_of::<char>() * 13;

    let word = BumpAllocator::qualloc::<char>(word_size as i32).unwrap() as *mut [char; 13];
    println!("Word address: {:p}", word);
    assert!(word as i32 > heap_address as i32);
    assert_eq!(word as i32, heap_address as i32 + BumpMemoryBlockHeader::size());

    BumpAllocator::qudelloc(word as *const ());

    #[cfg(not(target_os = "macos"))]
    {
        let current_heap = get_current_heap();
        assert_eq!(current_heap as i32, heap_address as i32);
    }
}
