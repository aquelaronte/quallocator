use std::sync::atomic::{AtomicPtr, Ordering};

use super::{
    MmapMemoryRegion, MmapMemorySectionHeader,
    globals::mmap_memory,
    utils::{allocate_region, deallocate_region, place_section_inside_region},
};

pub struct MmapAllocator {}

impl MmapAllocator {
    pub fn allocate<T>(size: usize) -> Option<*mut T> {
        let mut memory_guard = mmap_memory.lock().unwrap();

        if memory_guard.is_none() {
            /*
             * Creates a region with no sections stored inside
             */
            let new_region = allocate_region(size);

            if new_region.is_none() {
                return None;
            }

            let new_region = new_region.unwrap();

            /*
             * Get the address of region + MmapMemoryRegion.size, because just after the region block we
             * are going to store the header of the head section
             */
            let section_addr =
                (new_region as usize + MmapMemoryRegion::size()) as *mut MmapMemorySectionHeader;

            unsafe {
                (*section_addr) = MmapMemorySectionHeader::new(size, false, None, None);
                (*new_region).head_section = Some(AtomicPtr::new(section_addr));
                (*memory_guard) = Some(AtomicPtr::new(new_region));
            }

            let usr_pointer = (section_addr as usize + MmapMemorySectionHeader::size()) as *mut T;

            return Some(usr_pointer);
        }

        let mut current_region = memory_guard.as_ref().map(|ptr| ptr.load(Ordering::SeqCst));
        let mut last_region: Option<*mut MmapMemoryRegion> = None;

        while let Some(region) = current_region {
            unsafe {
                last_region = Some(region);
                if (*region).space_available < size {
                    current_region = region
                        .as_ref()
                        .unwrap()
                        .next
                        .as_ref()
                        .map(|ptr| ptr.load(Ordering::SeqCst));
                    continue;
                }

                let section = place_section_inside_region(region, size);

                if section.is_none() {
                    current_region = region
                        .as_ref()
                        .unwrap()
                        .next
                        .as_ref()
                        .map(|ptr| ptr.load(Ordering::SeqCst));
                    continue;
                }

                let section = section.unwrap();
                let usr_ptr = section as usize + MmapMemorySectionHeader::size();

                return Some(usr_ptr as *mut T);
            }
        }

        /*
         * If there aren't regions that can store the user data, then we must allocate a new one
         */
        let new_region = allocate_region(size);

        if new_region.is_none() {
            return None;
        }

        let new_region = new_region.unwrap();

        let section_addr = place_section_inside_region(new_region, size);

        /*
         * If for any reason, section can't be stored y the new_region, then we must abort and revert all
         */
        if section_addr.is_none() {
            deallocate_region(new_region);
            return None;
        }

        let section_addr = section_addr.unwrap();

        /*
         * If allocation was succesful, the we must push the region at the end of the list by
         * adding previous pointer of the new_region to points into last_region, and making
         * next pointer of the last_region to points into new_region
         */
        unsafe {
            (*new_region).prev = last_region.map(|ptr| AtomicPtr::new(ptr));

            if let Some(last_region) = last_region {
                (*last_region).next = Some(AtomicPtr::new(new_region));
            }
        }

        let usr_pointer = (section_addr as usize + MmapMemorySectionHeader::size()) as *mut T;

        return Some(usr_pointer);
    }
}
