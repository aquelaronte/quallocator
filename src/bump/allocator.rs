use std::sync::atomic::{AtomicPtr, Ordering};

use super::{
    globals::bump_memory,
    utils::{allocate_block, deallocate_block},
};

pub struct BumpAllocator {}

impl BumpAllocator {
    /**
     * Allocate memory on the heap using the bump allocator.
     *
     * @param size The size of the memory to allocate.
     * @return A pointer to the allocated memory.
     *
     * @note This function is thread-safe.
     * @warning This function may return None if the system runs out of memory.
     * @warning A generic type must be provided to ensure proper alignment
     * if the type isn't provided, the qualloc function will assume the type is ()
     */
    pub fn qualloc<T>(size: i32) -> Option<*mut T> {
        let mut memory_guard = bump_memory.lock().unwrap();

        /*
         * If memory isn't initialized, allocate a new block of memory and assign it to the memory guard
         */
        if memory_guard.is_none() {
            unsafe {
                let old_break = allocate_block::<T>(size)?;

                if memory_guard.is_none() {
                    *memory_guard = Some(AtomicPtr::new(old_break));
                }

                let user_ptr = old_break.add(1);

                return Some(user_ptr as *mut T);
            }
        }

        /*
         * If memory is initialized, search for a free block of memory that is large enough to allocate the requested memory
         */
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

                return Some(user_ptr as *mut T);
            }
        }

        /*
         * If no free block of memory is found, allocate a new block of memory
         */
        let old_break = allocate_block::<T>(size)?;

        unsafe {
            let user_ptr = old_break.add(1);

            return Some(user_ptr as *mut T);
        }
    }

    /**
     * Deallocate memory on the heap using the bump allocator.
     *
     * @param usr_data The pointer to the memory to deallocate.
     *
     * @note This function is thread-safe.
     */
    pub fn qudelloc(usr_data: *const ()) {
        let mut memory_guard = bump_memory.lock().unwrap();

        /*
         * If memory isn't initialized, do nothing
         */
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
}
