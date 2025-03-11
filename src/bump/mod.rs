use std::sync::atomic::AtomicPtr;

pub mod globals;
pub mod qualloc;
pub mod utils;
pub mod qudelloc;

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
