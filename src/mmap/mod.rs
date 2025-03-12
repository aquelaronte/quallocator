use std::sync::atomic::AtomicPtr;

pub mod globals;
pub mod utils;

/**
 * Mmap memory allocator is the modern way to make a memory allocator, it uses mmap and unmap
 *
 * The structure for dynamic data stored using mmap is the following
 *
 * Example:
 *
 * We need to store 4 bytes of data from user (a character), so we are going to use mmap to ask OS for
 * a block of memory, the block of memory size must be a multiplier of your page size, this block is commonly
 * called "Region", so we store the 4 bytes of data of the user into a part of our Region, so if user needs
 * to store more data, we put all the data from the user into a block
 *
 * When a block haves all of it's parts empty, we must call unmap for deallocating the memory block and returning
 * that memory to the OS
 *
 * Consider this case:
 *
 * User are going to store a String, "Hello World!" (48 bytes), so, we use mmap and OS gives this to us
 *
 * __________________________________________
 * |                                        |
 * |                                        |
 * |               1024 bytes               |
 * |                                        |
 * |                                        |
 * __________________________________________
 *
 * With this block of memory, we need to manage the data for storing all that user needs, but before, we are
 * going to store some structures into that block allowing the system to manage it, so the block will take this
 * form
 *
 * __________________________________________
 * |         |                               |
 * |         |                               |
 * | Region  |   1024 bytes - region size    |
 * |         |       of free space           |
 * |         |                               |
 * __________________________________________
 *
 * With this structure, we are going to split the free space from region for storing the User String
 *
 * __________________________________________
 * |         |                 |             |
 * |         |                 | 1024 bytes  |
 * | Region  | "Hello World!"  |  - region   |
 * |         |                 |  - string   |
 * |         |                 |             |
 * __________________________________________
 *
 * So, we already haves a block of memory and we are storing the user data into it, if the user needs to store more
 * stuff, it will be placed into the block free space, if block doesn't have enough space then we must create other
 * 1024 bytes blocka (or, other block with more kb)
 *
 * __________________________________________
 * |         |                              |
 * |         |                              |
 * | Region  |        Ocupped data          |
 * |    1    |                              |
 * |         |                              |
 * __________________________________________
 * __________________________________________
 * |         |                               |
 * |         |                               |
 * | Region  |   1024 bytes - region size    |
 * |   2     |       of free space           |
 * |         |                               |
 * __________________________________________
 *
 * You can see the regions as a linked list, because all of these haves a pointer to the next region, and the small amounts
 * of memory into each region as another linked list, like this
 *
 * __________________________________________
 * |         | __________ __________         |
 * |         | |        | |        |         |
 * | Region  | |   1    | |   2    | ...     |
 * |         | |        | |        |         |
 * |         | _________  __________         |
 * __________________________________________
 *
 * Regions are a linked list between them, and memory splits are also a linked list between them
 */
pub struct MmapMemoryRegion {
    pub size: usize,
    pub head_block: Option<AtomicPtr<MmapMemoryBlockHeader>>,
    pub next: Option<AtomicPtr<MmapMemoryRegion>>,
    pub prev: Option<AtomicPtr<MmapMemoryRegion>>,
}

impl MmapMemoryRegion {
    pub fn new(
        size: usize,
        head_block: Option<AtomicPtr<MmapMemoryBlockHeader>>,
        next: Option<AtomicPtr<MmapMemoryRegion>>,
        prev: Option<AtomicPtr<MmapMemoryRegion>>,
    ) -> Self {
        Self {
            size,
            head_block,
            next,
            prev,
        }
    }

    pub fn size() -> usize {
        size_of::<Self>()
    }
}

pub struct MmapMemoryBlockHeader {
    pub size: usize,
    pub is_free: bool,
    pub next: Option<AtomicPtr<MmapMemoryBlockHeader>>,
    pub prev: Option<AtomicPtr<MmapMemoryBlockHeader>>,
}

impl MmapMemoryBlockHeader {
    pub fn new(
        size: usize,
        is_free: bool,
        next: Option<AtomicPtr<MmapMemoryBlockHeader>>,
        prev: Option<AtomicPtr<MmapMemoryBlockHeader>>,
    ) -> Self {
        Self {
            size,
            is_free,
            next,
            prev,
        }
    }

    pub fn size() -> usize {
        size_of::<Self>()
    }
}
