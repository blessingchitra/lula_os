mod alloc_buddy;
use core::{
    alloc::{GlobalAlloc, Layout}, 
    ptr::NonNull,
    cell::UnsafeCell,
    ptr::null_mut,
};

// use crate::sync::SpinLock;
use alloc_buddy::BuddyAllocator;

/// Similar to `core::alloc::GlobalAlloc` except the
/// `allocate` and `deallocate` take a mutable ref to self.
pub trait  Allocatable{
    fn allocate(&mut self, size: usize) -> Option<NonNull<u8>>;
    fn deallocate(&mut self, ptr: NonNull<u8>, size: usize);
}


pub struct Allocator <T: Allocatable> {
    allocator: UnsafeCell<T>,
}

/// TODO: BADDD: Use lock to get the underlying `allocator`
/// and better error & param checking. Here we just getting the ball rolling.
unsafe impl <T: Allocatable> GlobalAlloc for Allocator<T> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let allocator = &mut *self.allocator.get();
        let size = layout.size();
        if let Some(address) = allocator.allocate(size){
            return address.as_ptr();
        }
        null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let allocator = &mut *self.allocator.get();
        let ptr = NonNull::new_unchecked(ptr);
        allocator.deallocate(ptr, layout.size());
    }
}

const fn create_config<T: Allocatable>() -> Allocator<T> {
    let base_size = 4096;
    let conn_multiplier = 4;
    
    Allocator{
        allocator: T::new(1000, 1024 * 1024 * 4).unwrap()
    }
}


static mut GLOB_ALLOCATOR: Allocator<BuddyAllocator> = 
Allocator{
    allocator: BuddyAllocator::new(1000, 1024 * 1024 * 4).unwrap()
};


pub fn allocator_init(){
    kprintln!("Memory Allocator initialsed");
}