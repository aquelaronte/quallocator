use lazy_static::lazy_static;
use std::sync::{Mutex, atomic::AtomicPtr};

use super::BumpMemoryBlockHeader;

lazy_static! {
    pub static ref bump_memory: Mutex<Option<AtomicPtr<BumpMemoryBlockHeader>>> = Mutex::new(None);
}
