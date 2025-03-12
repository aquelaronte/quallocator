use std::sync::{Mutex, atomic::AtomicPtr};

use lazy_static::lazy_static;

use super::MmapMemoryRegion;

lazy_static! {
    pub static ref mmap_memory: Mutex<Option<AtomicPtr<MmapMemoryRegion>>> = Mutex::new(None);
}
