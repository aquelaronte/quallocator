use std::sync::atomic::{AtomicPtr, Ordering};

use super::{globals::bump_memory, utils::allocate_block};

pub fn qualloc(size: i32) -> Option<*mut ()> {
    let mut memory_guard = bump_memory.lock().unwrap();

    if memory_guard.is_none() {
        unsafe {
            let old_break = allocate_block(size)?;

            if memory_guard.is_none() {
                *memory_guard = Some(AtomicPtr::new(old_break));
            }

            let user_ptr = old_break.add(1);

            return Some(user_ptr as *mut ());
        }
    }

    let mut current_node = memory_guard.as_ref().map(|ptr| ptr.load(Ordering::SeqCst));

    while let Some(node) = current_node {
        unsafe {
            if !(*node).is_free {
                current_node = (*node).next.as_ref().map(|ptr| ptr.load(Ordering::SeqCst));
                continue;
            }

            if (*node).size < size {
                current_node = (*node).next.as_ref().map(|ptr| ptr.load(Ordering::SeqCst));
                continue;
            }

            (*node).is_free = false;
            let user_ptr = node.add(1);

            return Some(user_ptr as *mut ());
        }
    }

    let old_break = allocate_block(size)?;

    unsafe {
        let user_ptr = old_break.add(1);

        return Some(user_ptr as *mut ());
    }
}
