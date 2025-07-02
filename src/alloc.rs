// src/alloc.rs

use std::alloc::{GlobalAlloc, Layout};
use std::os::raw::c_void;

pub struct HardenedAlloc;

unsafe impl GlobalAlloc for HardenedAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Calling the malloc function declared in the libc crate.
        // At final link time, this will resolve to the malloc implementation
        // in libhardened_malloc.a
        libc::malloc(layout.size()) as *mut u8
    }

    // `dealloc` method, corresponding to C's free.
    // The `_layout` parameter is prefixed with an underscore to silence the unused variable warning.
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        libc::free(ptr as *mut c_void)
    }

    // `realloc` method, corresponding to C's realloc.
    // The `layout` parameter is prefixed with an underscore as it's required by the trait
    // but not used by the underlying C function.
    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        libc::realloc(ptr as *mut c_void, new_size) as *mut u8
    }

    // `alloc_zeroed` method, corresponding to C's calloc.
    // Overriding this is a best practice for efficiency.
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // calloc(number_of_items, size_of_each_item) is the correct signature.
        // We allocate 1 item of layout.size() bytes.
        libc::calloc(1, layout.size()) as *mut u8
    }
}
