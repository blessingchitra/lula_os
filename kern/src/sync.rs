use core::sync::atomic::{AtomicUsize, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut, Drop};


/// A very simple spinlock
/// 
/// Example usage -------
/// ```
/// #[derive(Debug)]
/// struct BuffExample {
///     buffer: [u8; 1024],
///     rd:     usize,
///     wt:     usize,
/// }
/// 
/// impl BuffExample {
///     pub const fn new() -> Self {
///         Self { 
///             buffer: [0; 1024], 
///             rd: 0, 
///             wt: 0 
///         }
///     }
/// }
/// 
/// static RX_BUFFER: SpinLock<BuffExample> = SpinLock::new(BuffExample::new());
/// 
/// fn process_buff() {
///     let mut guard = RX_BUFFER.lock();
///     let buff = guard.get_mut();
///     write_five_to_buff(buff); 
/// }
/// 
/// 
/// fn write_five_to_buff(buff: &mut BuffExample) {
///     let _ = buff.buffer[buff.buffer.len() - 1];
///     buff.buffer[buff.buffer.len() - 1] = 5u8;
/// }
/// ```

#[derive(Debug)]
#[repr(C, align(8))]
pub struct SpinLock <T>{
    key:     AtomicUsize,
    hard_id: Option<usize>,
    data:    UnsafeCell<T>,
}

unsafe impl <T: Send> Send for SpinLock<T> {}
unsafe impl <T: Sync> Sync for SpinLock<T> {}

pub struct SpinLockGuard <'a, T> {
    lock: &'a SpinLock<T>,
    irq_enabled: bool,
}

impl <'a, T> SpinLockGuard<'a, T>{
    pub fn get_mut(&mut self) -> &mut T{
        self.deref_mut()
    }
    pub fn get(&self) -> &T{
        self.deref()
    }
}

impl <'a, T> Drop for SpinLockGuard <'a, T>{
    fn drop(&mut self){
        self.lock.key.store(0, Ordering::Release);
        if self.irq_enabled {
            crate::riscv::intr_on();
        }
    }
}

impl <'a, T> Deref for SpinLockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {& *(self.lock.data.get()) }
    }
}

impl <'a, T> DerefMut for SpinLockGuard<'a, T>{
    fn deref_mut(&mut self) -> &mut T {
        unsafe {&mut *(self.lock.data.get())}
    }
}

impl <T> SpinLock <T>{
    pub const fn new(data: T) -> Self{
        Self {
            key: AtomicUsize::new(0),
            hard_id: None,
            data: UnsafeCell::new(data)
        }
    }

    pub fn lock(&self) -> SpinLockGuard<'_,T>{
        let irq_enabled = crate::riscv::intr_get();
        crate::riscv::intr_off();
        while self.key.compare_exchange(
            0, 1, 
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_err() { core::hint::spin_loop(); }

        SpinLockGuard {
            lock: self,
            irq_enabled
        }
    }
}