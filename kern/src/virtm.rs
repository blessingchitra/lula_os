use core::sync::atomic::{AtomicU64, Ordering};

const PAGE_SIZE:  usize = 4096;
const BITMAP_LEN: usize = 64;
const PG_OFFSET:  usize = 12;

pub struct KPageAllocator {
    pmap:        &'static mut [AtomicU64],
    alloc_start: usize,
    page_count:  usize,
}

impl KPageAllocator {
    pub fn new(mem_start: usize, size: usize) -> Self {
        let page_count = size / PAGE_SIZE;
        let num_bitmaps = (page_count + (BITMAP_LEN - 1)) / BITMAP_LEN;

        let map_mem = mem_start as *mut AtomicU64;
        let pmap = unsafe {
            core::slice::from_raw_parts_mut(map_mem, num_bitmaps) };

        // TODO: Maybe we reserve the first page altogether for the page map ?
        let memory = mem_start + (num_bitmaps * core::mem::size_of::<AtomicU64>());
        Self {
            pmap,
            alloc_start: memory,
            page_count
        }
    }

    pub fn allocate(&mut self) -> Option<*mut u8>{
        for (idx, item) in self.pmap.iter().enumerate(){
            let map = item.load(Ordering::Relaxed);
            if map != !0 {
                let bit_idx =  map.leading_ones() as usize;
                if bit_idx > BITMAP_LEN {continue;}
                let mask = 1u64 << bit_idx;

                item.fetch_or(mask, Ordering::SeqCst);

                let offset = (BITMAP_LEN * idx) + bit_idx;
                let page = (self.alloc_start + (PAGE_SIZE * offset)) as *mut u8;
                return Some(page);
            }
        }
        return None;
    }

    pub fn deallocate(&mut self, addr: *mut u8){
        let addr = addr as usize;
        let invalid_addr = addr < self.alloc_start || (addr % PAGE_SIZE) != 0;
        if invalid_addr || addr >= (self.alloc_start + self.page_count * PAGE_SIZE) {
            return;
        }

        let page_idx = (addr - self.alloc_start) / PAGE_SIZE;
        let map_idx = page_idx / BITMAP_LEN;
        let bit_idx = page_idx % BITMAP_LEN;
        let map = &mut self.pmap[map_idx];

        let mask = 1u64 << bit_idx;
        map.fetch_and(!mask, Ordering::AcqRel);
    }

    pub fn page_allocated(&self, addr: *mut u8) -> bool{
        let addr = addr as usize;
        let invalid_addr = addr < self.alloc_start || (addr % PAGE_SIZE) != 0;
        if invalid_addr || addr >= (self.alloc_start + self.page_count * PAGE_SIZE) {
            return false;
        }
        
        let page_idx = (addr - self.alloc_start) / PAGE_SIZE;
        let map_idx = page_idx / BITMAP_LEN;
        let bit_idx = page_idx % BITMAP_LEN;

        let map = self.pmap[map_idx].load(Ordering::Relaxed);
        let mask = 1u64 << bit_idx;
        (map & mask) != 0
    }
}

#[allow(unused)]
fn test(){
    const LEN: usize = (1024 * 1024) / 8;
    let mut memory = [0u64; LEN];

    let mut allocator = KPageAllocator::new(memory.as_mut_ptr() as usize, LEN);
    
    let allocated_arr = allocator.allocate().unwrap();
    assert!(allocator.page_allocated(allocated_arr));

    allocator.deallocate(allocated_arr);
    assert!(!allocator.page_allocated(allocated_arr))
}
