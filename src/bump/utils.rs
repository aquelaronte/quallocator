use std::sync::atomic::{AtomicPtr, Ordering};

use super::{BumpMemoryBlockHeader, globals::bump_memory};
use libc::sbrk;

pub fn align_up(size: i32) -> i32 {
    (size + (8 - 1)) & !(8 - 1)
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
        let aligned_user_data_size = align_up(size);
        let allocated_size = BumpMemoryBlockHeader::size() + aligned_user_data_size;

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

/**
 * Get the adjacent free blocks of the current block.
 *
 * @param current_block The current block of memory.
 * @param stop_size The size of the block to stop the search for adjacent blocks.
 * @return A tuple containing first the merged block and then the last scanned block used for optimization
 *
 * @note This function is unsafe and should only be called by the bump allocator.
 *
 * Two blocks are adjacent if this condition is met:
 *
 * pt1 = Block allocated in 00 position of memory
 * pt2 = block allocated in 20 position of memory
 *
 * pt1.size + BumpMemoryBlockHeader::size() == pt2
 *
 * Blocks are adjacent because one is allocated next to the other in memory, because memory can be fragmented, if
 * pt2 is in 23 position of memory and pt1.size + BumpMemoryBlockHeader::size() == 20, then they are not adjacent because
 * other data is between them
 *
 * Example for adjacent blocks:
 * __________________________
 * |         pt1           | <-- This can be in position 00 - 19
 * __________________________
 * |         pt2           | <-- This can be in position 20 - 39
 * __________________________
 *
 * Example for non-adjacent blocks:
 * __________________________
 * |         pt1           | <-- This can be in position 00 - 19
 * __________________________
 * |some other data in heap| <-- This can be in position 20 - 25
 * __________________________
 * |         pt2           | <-- This can be in position 26 - 45
 * __________________________
 *
 * If this function mets adjacent blocks, then will merge them and return the pointer to a BumpMemoryBlockHeader with the size of
 * the sum of the sizes of the two blocks.
 *
 * Example for merged blocks:
 * __________________________
 * |         pt1           | <-- This can be in position 00 - 19
 * __________________________
 * |         pt2           | <-- This can be in position 20 - 39
 * __________________________
 *
 * Merged blocks:
 * __________________________
 * |         merged        | <-- This can be in position 00 - 39
 * __________________________
 *
 * The size of the merged blocks must be greater or equal than the stop_size, if not, then the function will return None
 */
pub fn merge_adjacent_free_blocks(
    initial_block: *mut BumpMemoryBlockHeader,
    stop_size: i32,
) -> (
    Option<*mut BumpMemoryBlockHeader>,
    Option<*mut BumpMemoryBlockHeader>,
) {
    let mut current_block = initial_block;
    let mut last_scanned_block: Option<*mut BumpMemoryBlockHeader> = None;
    let mut acumulated_size = 0;

    unsafe {
        if (*current_block).is_free {
            acumulated_size += (*current_block).size;
        }

        while (*current_block).is_free {
            last_scanned_block = Some(current_block);

            if acumulated_size >= stop_size {
                break;
            }

            let next_block = (*current_block)
                .next
                .as_ref()
                .map(|ptr| ptr.load(Ordering::SeqCst));

            /*
             * Check if the next block is free and if it is, then we must check if it is adjacent to the current block
             */
            if let Some(next_block) = next_block {
                let next_block_address = next_block as i32;
                let current_block_address = current_block as i32;
                let current_block_size = (*current_block).size;

                /*
                 * If block is adjacent, then we must add to acumulated size all the ocupped size
                 * by the next pointer (header and size attribute)
                 */
                if (*next_block).is_free
                    && (current_block_address + BumpMemoryBlockHeader::size() + current_block_size)
                        == next_block_address
                {
                    acumulated_size += (*current_block).size + BumpMemoryBlockHeader::size();
                }

                current_block = next_block;
            } else {
                break;
            }
        }
    }

    /*
     * If the size of the merged blocks is less than the stop size, then return None,
     * and the second element of the tuple will be the last scanned block because it is useful for allocating for skipping
     * the blocks that are smaller than the stop size.
     */
    if acumulated_size < stop_size {
        return (None, last_scanned_block);
    }

    /*
     * Update the size of the current block and the next pointer.
     *
     * The merged blocks next pointer should pointer to the block after the last scanned block.
     *
     * Example:
     * __________________________
     * |         pt1           | <-- This block is free, can be in position 00 - 19 and points to pt2
     * __________________________
     * |         pt2           | <-- This block is free, can be in position 20 - 39 and points to pt3
     * __________________________
     * |         pt3           | <-- This block isn't free, can be in position 40 - 59
     * __________________________
     *
     * After merging:
     * __________________________
     * |         merged        | <-- This block is free, can be in position 00 - 59 and points to pt3
     * __________________________
     * |         pt3           | <-- This block isn't free, can be in position 40 - 59
     * __________________________
     *
     */
    unsafe {
        (*initial_block).size = acumulated_size;

        if let Some(last_scanned_block) = last_scanned_block {
            if let Some(next_block) = (*last_scanned_block)
                .next
                .as_ref()
                .map(|ptr| ptr.load(Ordering::SeqCst))
            {
                (*initial_block).next = Some(AtomicPtr::new(next_block));
                (*next_block).prev = Some(AtomicPtr::new(initial_block));
            } else {
                (*initial_block).next = None;
            }
        } else {
            (*initial_block).next = None;
        }
    }

    return (Some(initial_block), last_scanned_block);
}

pub fn scan_bump_memory() {
    unsafe {
        let memory_guard = bump_memory.lock().unwrap();

        println!("Bump memory scanning results:");
        if memory_guard.is_none() {
            println!("Bump memory is empty");
            return;
        }
        let mut current_node = memory_guard.as_ref().map(|ptr| ptr.load(Ordering::SeqCst));

        while let Some(node) = current_node {
            println!(
                "{:p}:\n\t- size: {} bytes\n\t- free: {}\n",
                node,
                (*node).size,
                (*node).is_free
            );
            current_node = (*node).next.as_ref().map(|ptr| ptr.load(Ordering::SeqCst));
        }

        println!("Bump memory end");
    }
}
