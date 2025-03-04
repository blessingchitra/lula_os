use core::ptr::NonNull;
use core::sync::atomic::{AtomicUsize, Ordering};
use super::{Allocatable, AllocatableConfig, AllocatableErr};


#[repr(C)]
struct FreeBlock {
    next: Option<NonNull<FreeBlock>>,
}

pub struct BuddyAllocator {
    base            : usize,
    size            : usize,
    free_lists      : [Option<NonNull<FreeBlock>>; BuddyAllocator::NUM_ORDERS],
    split_bitmap    : [u8; BuddyAllocator::BITMAP_SIZE],
    allocated_bitmap: [u8; BuddyAllocator::BITMAP_SIZE],
    stats           : AllocatorStats,
}

struct AllocatorStats {
    allocated_bytes     : AtomicUsize,
    total_allocations   : AtomicUsize,
    total_deallocations : AtomicUsize,
}

impl BuddyAllocator {
    pub const MAX_ORDER     : usize = 17;               // 128KB max block size
    pub const MIN_ORDER     : usize = 5;                // 32 bytes min block size
    pub const NUM_ORDERS    : usize = BuddyAllocator::MAX_ORDER - BuddyAllocator::MIN_ORDER + 1;
    pub const MIN_BLOCK_SIZE: usize = 1 << BuddyAllocator::MIN_ORDER;
    pub const MAX_MEMORY    : usize = 1024 * 1024 * 100; // 100MB maximum managed memory
    pub const BITMAP_SIZE   : usize = (BuddyAllocator::MAX_MEMORY / BuddyAllocator::MIN_BLOCK_SIZE + 7) / 8;

    fn init(&mut self) {
        let mut remaining_size = self.size;
        let mut current_addr   = self.base;

        for order in (BuddyAllocator::MIN_ORDER..=BuddyAllocator::MAX_ORDER).rev() {
            let block_size = 1 << order;
            
            while remaining_size >= block_size && current_addr % block_size == 0 {
                let num_min_blocks = block_size / BuddyAllocator::MIN_BLOCK_SIZE;
                let start_index    = self.block_to_index(current_addr);
                self.set_allocated_range(start_index, num_min_blocks, false);
                
                unsafe {
                    let block_ptr     = current_addr as *mut FreeBlock;
                    (*block_ptr).next = self.free_lists[order - BuddyAllocator::MIN_ORDER];
                    self.free_lists[order - BuddyAllocator::MIN_ORDER] = NonNull::new(block_ptr);
                }
                
                remaining_size -= block_size;
                current_addr += block_size;
            }
        }
    }

    fn block_to_index(&self, addr: usize) -> usize {
        (addr - self.base) / BuddyAllocator::MIN_BLOCK_SIZE
    }

    fn set_allocated_range(&mut self, start_index: usize, num_blocks: usize, value: bool) {
        for index in start_index..start_index + num_blocks {
            // self.set_bit(&mut self.allocated_bitmap, index, value);
            if index >= BuddyAllocator::MAX_MEMORY / BuddyAllocator::MIN_BLOCK_SIZE {
                return;
            }
            let byte_index = index / 8;
            let bit_index  = index % 8;
            if value {
                self.allocated_bitmap[byte_index] |= 1 << bit_index;
            } else {
                self.allocated_bitmap[byte_index] &= !(1 << bit_index);
            }
        }
    }

    fn set_split_bit(&mut self, index: usize, value: bool) {
        // self.set_bit(&mut self.split_bitmap, index, value);
        if index >= BuddyAllocator::MAX_MEMORY / BuddyAllocator::MIN_BLOCK_SIZE {
            return;
        }
        let byte_index = index / 8;
        let bit_index  = index % 8;
        if value {
            self.split_bitmap[byte_index] |= 1 << bit_index;
        } else {
            self.split_bitmap[byte_index] &= !(1 << bit_index);
        }
    }

    fn get_split_bit(&self, index: usize) -> bool {
        self.get_bit(&self.split_bitmap, index)
    }

    fn is_range_free(&self, start_index: usize, num_blocks: usize) -> bool {
        for i in start_index..start_index + num_blocks {
            if self.get_bit(&self.allocated_bitmap, i) {
                return false;
            }
        }
        true
    }

    fn set_bit(&mut self, bitmap: &mut [u8], index: usize, value: bool) {
        if index >= BuddyAllocator::MAX_MEMORY / BuddyAllocator::MIN_BLOCK_SIZE {
            return;
        }
        let byte_index = index / 8;
        let bit_index  = index % 8;
        if value {
            bitmap[byte_index] |= 1 << bit_index;
        } else {
            bitmap[byte_index] &= !(1 << bit_index);
        }
    }

    fn get_bit(&self, bitmap: &[u8], index: usize) -> bool {
        if index >= BuddyAllocator::MAX_MEMORY / BuddyAllocator::MIN_BLOCK_SIZE {
            return false;
        }
        let byte_index = index / 8;
        let bit_index  = index % 8;
        (bitmap[byte_index] & (1 << bit_index)) != 0
    }


    fn size_to_order(&self, size: usize) -> Option<usize> {
        let adjusted_size = size.max(BuddyAllocator::MIN_BLOCK_SIZE);
        let order         = adjusted_size.next_power_of_two().trailing_zeros() as usize;
        if order >= BuddyAllocator::MIN_ORDER && order <= BuddyAllocator::MAX_ORDER {
            Some(order)
        } else {
            None
        }
    }

    fn allocate_order(&mut self, order: usize) -> Option<NonNull<u8>> {
        if order < BuddyAllocator::MIN_ORDER || order > BuddyAllocator::MAX_ORDER {
            return None;
        }

        let index          = order - BuddyAllocator::MIN_ORDER;
        let block_size     = 1 << order;
        let num_min_blocks = block_size / BuddyAllocator::MIN_BLOCK_SIZE;

        if let Some(block)  = self.free_lists[index].take() {
            let block_addr  = block.as_ptr() as usize;
            let start_index = self.block_to_index(block_addr);
            self.set_allocated_range(start_index, num_min_blocks, true);
            
            self.stats.allocated_bytes.fetch_add(block_size, Ordering::Relaxed);
            self.stats.total_allocations.fetch_add(1, Ordering::Relaxed);
            return Some(unsafe { NonNull::new_unchecked(block.as_ptr() as *mut u8) });
        }

        if let Some(larger_block) = self.allocate_order(order + 1) {
            let larger_block_addr = larger_block.as_ptr() as usize;
            let parent_index      = self.block_to_index(larger_block_addr) / 2;
            self.set_split_bit(parent_index, true);

            let buddy_addr  = larger_block_addr + (1 << order);
            let buddy_index = self.block_to_index(buddy_addr);
            self.set_allocated_range(buddy_index, num_min_blocks, false);

            unsafe {
                let buddy_ptr = NonNull::new_unchecked(buddy_addr as *mut FreeBlock);
                (*buddy_ptr.as_ptr()).next = self.free_lists[index];
                self.free_lists[index] = Some(buddy_ptr);
            }
            
            return Some(larger_block);
        }

        None
    }


    fn deallocate_order(&mut self, ptr: NonNull<u8>, order: usize) {
        if order < BuddyAllocator::MIN_ORDER || order > BuddyAllocator::MAX_ORDER {
            return;
        }

        let block_addr     = ptr.as_ptr() as usize;
        let index          = order - BuddyAllocator::MIN_ORDER;
        let block_size     = 1 << order;
        let num_min_blocks = block_size / BuddyAllocator::MIN_BLOCK_SIZE;
        let start_index    = self.block_to_index(block_addr);

        self.set_allocated_range(start_index, num_min_blocks, false);

        self.stats.allocated_bytes.fetch_sub(block_size, Ordering::Relaxed);
        self.stats.total_deallocations.fetch_add(1, Ordering::Relaxed);

        if order < BuddyAllocator::MAX_ORDER {
            let parent_index = start_index / (2 * num_min_blocks);
            let buddy_addr = if (start_index / num_min_blocks) % 2 == 0 {
                block_addr + block_size
            } else {
                block_addr - block_size
            };

            let buddy_start_index = self.block_to_index(buddy_addr);
            
            if self.is_range_free(buddy_start_index, num_min_blocks) {
                self.remove_from_free_list(buddy_addr, index);
                self.set_split_bit(parent_index, false);
                let merged_addr = block_addr.min(buddy_addr);
                let merged_ptr  = unsafe { NonNull::new_unchecked(merged_addr as *mut u8) };
                self.deallocate_order(merged_ptr, order + 1);
                return;
            }
        }

        let block_ptr = block_addr as *mut FreeBlock;
        unsafe {
            (*block_ptr).next = self.free_lists[index];
            self.free_lists[index] = NonNull::new(block_ptr);
        }
    }

    fn remove_from_free_list(&mut self, addr: usize, index: usize) {
        let mut prev:Option<NonNull<FreeBlock>>     = None;
        let mut current = self.free_lists[index];

        while let Some(node) = current {
            if node.as_ptr() as usize == addr {
                if let Some(prev_node) = prev {
                    unsafe { (*prev_node.as_ptr()).next = node.as_ref().next };
                } else {
                    self.free_lists[index] = unsafe {node.as_ref().next};
                }
                return;
            }
            prev = current;
            current = unsafe { node.as_ref().next };
        }
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        (
            self.stats.allocated_bytes.load(Ordering::Relaxed),
            self.stats.total_allocations.load(Ordering::Relaxed),
            self.stats.total_deallocations.load(Ordering::Relaxed),
        )
    }
}


impl Allocatable for BuddyAllocator {
    fn new(config: AllocatableConfig) -> Result<Self, AllocatableErr> {
        let aligned_start = (config.start + BuddyAllocator::MIN_BLOCK_SIZE - 1) & !(BuddyAllocator::MIN_BLOCK_SIZE - 1);
        let adjusted_size = config.size - (aligned_start - config.start);
        let usable_size   = adjusted_size & !(BuddyAllocator::MIN_BLOCK_SIZE - 1);

        if usable_size > BuddyAllocator::MAX_MEMORY {
            return Err(AllocatableErr::NotEnoughMemory);
        }

        let mut allocator = BuddyAllocator {
            base            : aligned_start,
            size            : usable_size,
            free_lists      : [None; BuddyAllocator::NUM_ORDERS],
            split_bitmap    : [0; BuddyAllocator::BITMAP_SIZE],
            allocated_bitmap: [0; BuddyAllocator::BITMAP_SIZE],
            stats: AllocatorStats {
                allocated_bytes    : AtomicUsize::new(0),
                total_allocations  : AtomicUsize::new(0),
                total_deallocations: AtomicUsize::new(0),
            },
        };

        allocator.init();
        Ok(allocator)
    }

    fn allocate(&mut self, size: usize) -> Option<NonNull<u8>> {
        let order = self.size_to_order(size)?;
        self.allocate_order(order)
    }

    fn deallocate(&mut self, ptr: NonNull<u8>, size: usize) {
        if let Some(order) = self.size_to_order(size) {
            self.deallocate_order(ptr, order);
        }
    }
}

