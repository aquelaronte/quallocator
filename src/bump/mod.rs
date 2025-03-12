use std::sync::atomic::AtomicPtr;

pub mod globals;
pub mod utils;
pub mod allocator;

/**
 * Bump memory allocator is the classic type of dynamic memory management using sbrk
 * 
 * It follow a structure where dynamic data has a header and we increases or decreases the heap size
 * 
 * For example:
 * 
 * _________________________
 * |       header 1        | <- Size 16
 * _________________________
 * _________________________
 * |                       |
 * |       user data       | <- Size 40
 * |                       |
 * _________________________
 * _________________________
 * |       header 2        |
 * _________________________
 * _________________________
 * |                       |
 * |       user data       | <- Size 32
 * |                       |
 * _________________________
 * 
 * In this case we have the data stored by the user, but just above there is an additional structure that contains
 * user's stored data properties such as the size, the pointer to next block, to previous block, etc.
 * 
 * In the given example, the heap size is 104, but if we deletes the second block, the header and the user data will be
 * deallocated and we must decrease the heap size, so if the heap size was 104 before, after deleting second block, the heap
 * size should be 58
 */

pub struct BumpMemoryBlockHeader {
    pub size: i32,
    pub is_free: bool,
    pub next: Option<AtomicPtr<BumpMemoryBlockHeader>>,
    pub prev: Option<AtomicPtr<BumpMemoryBlockHeader>>,
}

impl BumpMemoryBlockHeader {
    pub fn new(
        size: i32,
        is_free: bool,
        next: Option<AtomicPtr<BumpMemoryBlockHeader>>,
        prev: Option<AtomicPtr<BumpMemoryBlockHeader>>,
    ) -> BumpMemoryBlockHeader {
        Self {
            next,
            is_free,
            prev,
            size,
        }
    }

    pub fn size() -> i32 {
        size_of::<BumpMemoryBlockHeader>() as i32
    }
}
