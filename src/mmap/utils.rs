use libc::{
    _SC_PAGESIZE, MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_READ, PROT_WRITE, mmap, munmap,
    sysconf,
};
use std::{
    os::raw::c_void,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

use super::{MmapMemoryBlockHeader, MmapMemoryRegion};

/**
 * Takes page size from the OS
 */
pub fn get_page_size() -> usize {
    unsafe { sysconf(_SC_PAGESIZE) as usize }
}

/**
 * Takes a number and rounds up to the closer page size multiplier
 *
 * example:
 * system page size = 1024
 * size = 1540
 *
 * rounded to page size = 2048
 */
pub fn round_up_to_page_size(size: usize) -> usize {
    let page_size = get_page_size();
    ((size + page_size - 1) / page_size) * page_size
}

/**
 * Allocates a region into heap, uses mmap for asking to OS a block of memory
 * and returns a pointer to the Region
 */
pub fn allocate_region(size: usize) -> Option<*mut MmapMemoryRegion> {
    let block_size = round_up_to_page_size(size + MmapMemoryRegion::size());

    let addr = unsafe {
        mmap(
            ptr::null_mut(),
            block_size,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        )
    } as *mut MmapMemoryRegion;

    if addr as *mut c_void == MAP_FAILED {
        return None;
    }

    unsafe {
        *addr = MmapMemoryRegion::new(block_size - MmapMemoryRegion::size(), None, None, None)
    }

    return Some(addr);
}

/**
 * Uses munmap for deallocating a block from the heap
 */
pub fn deallocate_region(region: *mut MmapMemoryRegion) {
    unsafe {
        munmap(region as *mut _, (*region).size + MmapMemoryRegion::size());
    }
}

/**
 * Takes a header and returns the last header from the linked list
 */
pub fn take_last_block(block: *mut MmapMemoryBlockHeader) -> *mut MmapMemoryBlockHeader {
    let mut current_block = block;

    unsafe {
        while (*current_block).next.is_some() {
            current_block = (*current_block)
                .next
                .as_mut()
                .unwrap()
                .load(Ordering::SeqCst);
        }
    }

    current_block
}

/**
 * Place a block inside a region
 */
pub fn place_block(
    region: *mut MmapMemoryRegion,
    size: usize,
) -> Option<*mut MmapMemoryBlockHeader> {
    unsafe {
        let real_size = size + MmapMemoryBlockHeader::size();

        /*
         * If region doesn't have enough space for store a new block, then it will return None
         */
        if (*region).size < real_size {
            return None;
        }

        /*
         * If region already has a head_block, then we must get the final node from the linked list and create an empty new Header
         * next to the last node and rest the node real_size on the region size
         */
        if let Some(head_block) = region.as_mut().unwrap().head_block.as_mut() {
            let head_block_ptr = head_block.load(Ordering::SeqCst);

            let last_block = take_last_block(head_block_ptr);

            /*
             * Gets the pointer next to the last_block
             */
            let block_pointer =
                (last_block.add(1) as usize + (*last_block).size) as *mut MmapMemoryBlockHeader;

            (*block_pointer) =
                MmapMemoryBlockHeader::new(size, false, None, Some(AtomicPtr::new(last_block)));

            (*region).size -= real_size;

            return Some(block_pointer);
        }

        /*
         * Gets the pointer next to the region header
         */
        let head_block = ((*region).size + MmapMemoryRegion::size()) as *mut MmapMemoryBlockHeader;

        *head_block = MmapMemoryBlockHeader::new(size, false, None, None);

        // Links the head_block attribute to the newly created block
        (*region).head_block = Some(AtomicPtr::new(head_block));

        (*region).size -= real_size;

        Some(head_block)
    }
}
