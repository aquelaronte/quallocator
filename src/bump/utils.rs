use super::BumpMemoryBlockHeader;
use libc::sbrk;

pub fn align_up<T>(size: i32) -> i32 {
    let system_alignment = align_of::<T>() as i32;
    (size + (system_alignment - 1)) & !(system_alignment - 1)
}

/**
 * Allocate a new block of memory for the bump allocator and set the header
 * for the new block.
 *
 * @param size The size of the new block of memory to allocate.
 * @return The pointer to the new block of memory.
 *
 * @note This function is unsafe and should only be called by the bump allocator.
 * @warning This function may return NULL if the system runs out of memory.
 */
pub fn allocate_block<T>(size: i32) -> Option<*mut BumpMemoryBlockHeader> {
    unsafe {
        // Add the size of the header to the size of the block
        let allocated_size = align_up::<T>(BumpMemoryBlockHeader::size() + size);
        let aligned_user_data_size = allocated_size - BumpMemoryBlockHeader::size();

        println!("Allocated size: {}", allocated_size);

        let old_break = sbrk(allocated_size) as *mut BumpMemoryBlockHeader;

        if old_break.is_null() {
            return None;
        }

        *old_break = BumpMemoryBlockHeader::new(aligned_user_data_size, false, None, None);

        Some(old_break)
    }
}

/**
 * Deallocate a block of memory for the bump allocator.
 *
 * @param size The size of the block of memory to deallocate.
 *
 * @note This function is unsafe and should only be called by the bump allocator.
 */
pub fn deallocate_block(size: i32) {
    unsafe {
        let deallocated_size = BumpMemoryBlockHeader::size() + size;
        println!("Dellocated size: {}", -deallocated_size);

        sbrk(-deallocated_size);
    }
}

/**
 * Get the current heap pointer.
 *
 * @return The current heap pointer.
 *
 * @note This function is unsafe and should only be called by the bump allocator.
 */
pub fn get_current_heap() -> *mut () {
    unsafe {
        return sbrk(0) as *mut ();
    }
}
