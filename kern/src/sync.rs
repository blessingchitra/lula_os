use core::sync::atomic::{AtomicBool, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut, Drop};


#[derive(Debug)]
pub struct SpinLock <T>{
    hard_id: Option<usize>,
    data:    UnsafeCell<T>,
    key:     AtomicBool,
}

unsafe impl <T: Send> Send for SpinLock<T> {}
unsafe impl <T: Sync> Sync for SpinLock<T> {}

pub struct SpinLockGuard <'a, T> {
    lock: &'a SpinLock<T>
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
        self.lock.key.store(false, Ordering::Release);
    }
}


impl <T> Deref for SpinLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe {& *(self.lock.data.get()) }
    }
}

impl <'a, T> DerefMut for SpinLockGuard<'a, T>{
    fn deref_mut(&mut self) -> &'a mut Self::Target {
        unsafe {&mut *(self.lock.data.get())}
    }
}

impl <T> SpinLock <T>{
    pub const fn new(data: T) -> Self{
        Self {
            key: AtomicBool::new(false),
            hard_id: None,
            data: UnsafeCell::new(data)
        }
    }

    pub fn lock(&self) -> SpinLockGuard<'_,T>{
        while self.key.compare_exchange(
            false, true, 
            Ordering::Acquire, Ordering::Relaxed).is_err() {
            core::hint::spin_loop();
        }
        SpinLockGuard {
            lock: self
        }
    }
}


impl <T> Deref for SpinLock<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {& *(self.data.get())}
    }
}

/// ----------
#[derive(Debug)]
struct KUartBuff {
    buffer: [u8; 1024],
    tx_ier: bool,
    rd:     usize,
    wt:     usize,
}

impl KUartBuff {
    pub const fn new() -> Self {
        Self { 
            buffer: [0; 1024], 
            tx_ier: false,
            rd: 0, 
            wt: 0 
        }
    }
}

static RX_BUFFER: SpinLock<KUartBuff> = SpinLock::new(KUartBuff::new());

fn process_buff() {
    let mut guard = RX_BUFFER.lock();
    let buff = guard.get_mut();
    write_five_to_buff(buff); 
}


fn write_five_to_buff(buff: &mut KUartBuff) {
    let _ = buff.buffer[buff.buffer.len() - 1];
    buff.buffer[buff.buffer.len() - 1] = 5u8;
}


























