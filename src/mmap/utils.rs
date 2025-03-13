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

/**
 * Gets a region and puts a section of memory inside it
 */
pub fn place_section_inside_region(
    region: *mut MmapMemoryRegion,
    size: usize,
) -> Option<*mut MmapMemorySectionHeader> {
    unsafe {
        /*
         * If regions doesn't have enough space to store the section of memory, then
         * we must return None
         */
        if (*region).space_available < size {
            return None;
        }

        let mut current_section = region
            .as_ref()
            .unwrap()
            .head_section
            .as_ref()
            .map(|ptr| ptr.load(Ordering::SeqCst));

        /*
         * In the case where region head_section isn't initialized, we must initialize it
         */
        if current_section.is_none() {
            // Gets the direction on memory just after the Region header
            let section_addr =
                (region as usize + MmapMemoryRegion::size()) as *mut MmapMemorySectionHeader;

            (*section_addr) = MmapMemorySectionHeader::new(size, false, None, None);
            (*region).head_section = Some(AtomicPtr::new(section_addr));
            (*region).space_available -= (*section_addr).size + MmapMemorySectionHeader::size();

            return Some(section_addr);
        }


        /*
         * If region is already initialized, then we must iterate over every child until we found
         * a section that is free and haves enough space for storing user data
         */
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

            /*
             * TODO: to implement a function for merging free adjacent sections, see more information
             * about merging adjacent space into [`super::bump::utils::merge_adjacent_free_blocks`]
             */
            if (*section).size < size {
                current_section = section
                    .as_ref()
                    .unwrap()
                    .next
                    .as_ref()
                    .map(|ptr| ptr.load(Ordering::SeqCst));
                continue;
            }

            /*
             * If free section with enough space is found, then we must set free to false and rest
             * the section space to the space_available plus SectionHeader
             */
            (*section).is_free = false;

            /*
             * At the moment of rest the space available, we must take in count the header size because
             * when we are deallocating space, we rest also the header space because it can be useful when 
             * we are merging adjacent blocks
             */
            (*region).space_available -= (*section).size + MmapMemorySectionHeader::size();

            return Some(section);
        }

        /*
         * If free blocks aren't found, then we must take the last section of the region and calculate this
         * 
         * First, we need to calculate the range of memory that a region haves
         * 
         * fr = first direction of memory that a region haves
         * lr = last direction of memory that a region haves
         * r = region pointer
         * 
         * fr = region
         * lr = fr + r.total_space + RegionHeader.size
         * 
         * Next, we need to check that if we place a section with the needed size by the user after the last section
         * of a region, it doesn't becomes greater than lr
         * 
         * For example: if the range of memory that a region haves is from 0x00 to 0xc0, and the last section is in 0xb0,
         * we must check that if we place a section with the given size after the last section, that user section doesn't
         * haves more size than the region range
         * 
         * s = size needed for the user
         * ls = last section of region
         * us = direction of memory where we are going to place the user section
         * 
         * us = ls + ls.size + SectionHeader.size
         * 
         * us + SectionHeader.size + s <= lr
         * 
         * If this condition is met, then we can place the user section just after the last section of region
         */

        None
    }
}
