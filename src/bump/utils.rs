use super::BumpMemoryBlockHeader;
use libc::sbrk;

pub fn allocate_block(size: i32) -> Option<*mut BumpMemoryBlockHeader> {
    unsafe {
        let allocated_size = BumpMemoryBlockHeader::size() + size;
        println!("Allocated size: {}", allocated_size);
        let old_break = sbrk(allocated_size) as *mut BumpMemoryBlockHeader;

        if old_break.is_null() {
            return None;
        }

        *old_break = BumpMemoryBlockHeader::new(size, false, None, None);

        Some(old_break)
    }
}

pub fn deallocate_block(size: i32) {
    unsafe {
        let deallocated_size = BumpMemoryBlockHeader::size() + size;
        println!("Dellocated size: {}", -deallocated_size);

        sbrk(-deallocated_size);
    }
}

pub fn get_current_heap() -> *mut () {
    unsafe {
        return sbrk(0) as *mut ();
    }
}
