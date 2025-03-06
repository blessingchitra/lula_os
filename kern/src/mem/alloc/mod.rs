mod alloc_buddy;
use core::{
    alloc::{GlobalAlloc, Layout}, 
    ptr::{NonNull, null_mut},
    cell::UnsafeCell,
};

// use crate::sync::SpinLock;
use alloc_buddy::BuddyAllocator;

use crate::sync::{self, SpinLock};

pub struct  AllocatableConfig {
    start: usize,
    size : usize,
}

const PAGE_SIZE   : usize = 4096;

pub const KERN_START  : usize = 0x80000000;
pub const KERN_RESERV : usize = 128 * (1024 * 1024);
pub const MEM_MAX : usize = 1usize << (9 + 9 + 9 + 12 - 1);

#[non_exhaustive]
#[derive(Debug)]
pub enum AllocatableErr{
    ExceedsAllocatableLimit,    // `Memory Request Error`. Memory size exceeds allocatable size
    ExceedsAllocatorMaxCap      // `Initialisation Error`. The Allocator cannot managange heap of x size
}

/// Similar to `core::alloc::GlobalAlloc` except the
/// `allocate` and `deallocate` take a mutable ref to self.
pub trait Allocatable{
    fn new(config: AllocatableConfig) -> Result<Self, AllocatableErr> where Self: Sized;
    fn allocate(&mut self, layout: Layout) -> Option<NonNull<u8>>;
    fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout);
}

pub struct Allocator <T: Allocatable> {
    allocator: Option<SpinLock<UnsafeCell<T>>>,
}

impl <T: Allocatable> Allocator <T> {
    pub fn get_allocator(&self) -> Option<&mut T> {
        if let Some(lock) = &self.allocator {
            let mut guard = lock.lock();
            let data = unsafe {&mut *guard.get_mut().get()};
            return Some(data);
        }
        None
    }

    pub fn set_allocator(&mut self, allocator: T){
        let alloc = 
                            sync::SpinLock::new(UnsafeCell::new(allocator));
        self.allocator = Some(alloc);
    }
}

/// TODO: Add better error & param checking.
unsafe impl <T: Allocatable> GlobalAlloc for Allocator<T> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Some(allocator) = self.get_allocator(){
            if let Some(address) = allocator.allocate(layout){
                return address.as_ptr();
            }
        }
        null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(allocator) = self.get_allocator(){
            let ptr = NonNull::new_unchecked(ptr);
            allocator.deallocate(ptr, layout);
        }
    }
}

#[inline]
fn get_end() -> usize {
    let x: usize;
    unsafe {
        core::arch::asm!(
            " la {}, end", out(reg) x,)};
    x
}

fn create_allocator<T: Allocatable>() -> T {

    let mut kern_end = get_end();
    kern_end = (kern_end + (PAGE_SIZE - 1)) & !(PAGE_SIZE - 1);
    let mem_size = KERN_RESERV - (kern_end - KERN_START);
    let config = AllocatableConfig{start: kern_end, size: mem_size};

    let allocator = T::new(config).expect("Could Not Initialise Memory Allocator");
    allocator
}

#[global_allocator]
static mut GLOB_ALLOCATOR: Allocator<BuddyAllocator> = Allocator{ allocator: None};

/// Initialisation
/// - This function gets run once by the CPU that initialised the
///     kernel.
pub fn allocator_init(){
    unsafe {
        let allocator = create_allocator::<BuddyAllocator>();
        GLOB_ALLOCATOR.set_allocator(allocator);
    }
    kprintln!("Memory Allocator initialsed");
}



