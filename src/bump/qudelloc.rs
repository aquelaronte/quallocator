use std::sync::atomic::Ordering;

use super::{globals::bump_memory, utils::deallocate_block};

pub fn qudelloc(usr_data: *const ()) {
    let mut memory_guard = bump_memory.lock().unwrap();

    if memory_guard.is_none() {
        return;
    }

    let mut current_node = memory_guard.as_ref().map(|ptr| ptr.load(Ordering::SeqCst));

    /*
     * Check if deallocated node is head node
     */
    if let Some(node) = current_node {
        unsafe {
            let usr_data_ptr = node.add(1) as *const ();

            if usr_data_ptr == usr_data {
                (*node).is_free = true;

                /*
                 * If deallocated node is head node and it's last node, we must decrease
                 * heap size, otherwise, just return after set it free
                 */
                if (*node).next.is_none() {
                    (*memory_guard) = None;
                    deallocate_block((*node).size);
                    return;
                }

                return;
            }
        }
    }

    while let Some(node) = current_node {
        unsafe {
            let usr_data_ptr = node.add(1) as *const ();

            if usr_data_ptr != usr_data {
                current_node = (*node).next.as_ref().map(|ptr| ptr.load(Ordering::SeqCst));
                continue;
            }

            (*node).is_free = true;

            if (*node).next.is_none() {
                if let Some(prev_ptr) = &(*node).prev {
                    let prev = prev_ptr.load(Ordering::SeqCst);

                    (*prev).next = None;
                }

                deallocate_block((*node).size);
            }
        }
    }
}
