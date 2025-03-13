use libc::{
    _SC_PAGESIZE, MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_READ, PROT_WRITE, mmap, munmap,
    sysconf,
};
use std::{
    os::raw::c_void,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

use super::{MmapMemoryRegion, MmapMemorySectionHeader};

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

    let stored_size = block_size - MmapMemorySectionHeader::size();

    /*
     * In region size we are going to store the memory block size minus Header Region size
     *
     * That means that if we have 1024 bytes of size in given block by mmap, then Region Header will store
     * in size attribute the value of that 1024 bytes minus the Region Header, so if Region Header weight
     * is about 30 bytes, then the region size attribute must mark 994 bytes of free space
     */
    unsafe { *addr = MmapMemoryRegion::new(stored_size, stored_size, None, None, None) }

    return Some(addr);
}

/**
 * Uses munmap for deallocating a block from the heap
 */
pub fn deallocate_region(region: *mut MmapMemoryRegion) {
    unsafe {
        /*
         * Region.total_space contains the block size without the region size itself
         */
        munmap(
            region as *mut _,
            (*region).total_space + MmapMemoryRegion::size(),
        );
    }
}

pub fn place_section_inside_region(
    region: *mut MmapMemoryRegion,
    size: usize,
) -> Option<*mut MmapMemorySectionHeader> {
    unsafe {
        if (*region).space_available < size {
            return None;
        }

        let mut current_section = region
            .as_ref()
            .unwrap()
            .head_section
            .as_ref()
            .map(|ptr| ptr.load(Ordering::SeqCst));

        if current_section.is_none() {
            let section_addr =
                (region as usize + MmapMemoryRegion::size()) as *mut MmapMemorySectionHeader;

            (*section_addr) = MmapMemorySectionHeader::new(size, false, None, None);
            (*region).head_section = Some(AtomicPtr::new(section_addr));
            (*region).space_available -= (*section_addr).size + MmapMemorySectionHeader::size();

            return Some(section_addr);
        }

        while let Some(section) = current_section {
            if !(*section).is_free {
                current_section = section
                    .as_ref()
                    .unwrap()
                    .next
                    .as_ref()
                    .map(|ptr| ptr.load(Ordering::SeqCst));
                continue;
            }

            if (*section).size < size {
                current_section = section
                    .as_ref()
                    .unwrap()
                    .next
                    .as_ref()
                    .map(|ptr| ptr.load(Ordering::SeqCst));
                continue;
            }

            (*section).is_free = false;
            (*region).space_available -= (*section).size;

            return Some(section);
        }

        None
    }
}
