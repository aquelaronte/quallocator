use crate::bump::{
    BumpMemoryBlockHeader,
    allocator::BumpAllocator,
    utils::{align_up, get_current_heap, scan_bump_memory},
};
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
    let aligned_size = align_up(13);
    let char_alignment = align_of::<char>() as i32;

    assert!(aligned_size % char_alignment == 0);

    /*
     * u8 alignment constant is 1 on x86_64, so every number is aligned
     * even is it's a prime number
     */
    let aligned_size = align_up(17);
    assert_eq!(aligned_size, 17);
}

#[test]
fn test_qualloc() {
    /*
     * In this test we are going to test if block are correctly reused
     * First, we must allocate a block with size 52
     * Next, we must allocate a second block with size 52 too
     * Next, we are freeing first block
     * Next, we must allocate other block with size 52
     *
     * The result must be that first block must be ocupped by the last allocated block
     */

    let initial_heap_address = get_current_heap() as i32;
    let aligned_size = align_up(52);

    // First block
    let first_block = BumpAllocator::qualloc::<char>(aligned_size).unwrap();
    scan_bump_memory();

    // Second block
    let second_block = BumpAllocator::qualloc::<char>(aligned_size).unwrap();
    scan_bump_memory();

    BumpAllocator::qudelloc(first_block);
    scan_bump_memory();

    // First block again
    let first_block_again = BumpAllocator::qualloc::<char>(aligned_size).unwrap();
    scan_bump_memory();

    // Asserts
    assert_eq!(
        first_block as i32, first_block_again as i32,
        "Third block must have the same address as the first block"
    );
    assert_eq!(
        second_block as i32,
        initial_heap_address + BumpMemoryBlockHeader::size() * 2 + aligned_size,
        "Second block must have the same direction as the first pointer plus it's size and the header size"
    );

    BumpAllocator::qualloc::<char>(aligned_size).unwrap();
    BumpAllocator::qudelloc(first_block_again);
    BumpAllocator::qudelloc(second_block);
    scan_bump_memory();
    let merged_two_blocks = BumpAllocator::qualloc::<char>(aligned_size * 2).unwrap();
    scan_bump_memory();

    assert_eq!(
        merged_two_blocks as i32 - BumpMemoryBlockHeader::size(),
        initial_heap_address,
        "Third block size must be equal to aligned_size * 2 (given size) plus header size (because deallocated blocks was merge)"
    );
}
