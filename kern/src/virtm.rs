use core::sync::atomic::{AtomicU64, Ordering};

const PAGE_SIZE   : usize = 4096;
const BITMAP_LEN  : usize = 64;
const PAGE_OFFSET : usize = 12;
const PAGE_FLAGS  : u8    = 10;
const LEVEL_MASK  : usize = 0x1FF;

const KERN_START  : usize = 0x80000000;
const KERN_RESERV : usize = 128 * (1024 * 1024);
const MEM_MAX     : usize = 1usize << (9 + 9 + 9 + 12 - 1);

pub static mut KERN_SATP: usize = 0;
pub static mut KERN_PG_ALLOCATOR: Option<KPageAllocator> = None;

pub struct KPageAllocator {
    pmap:        &'static mut [AtomicU64],
    alloc_start: usize,
    page_count:  usize,
}

pub enum AllocErr{
    AddrNotAligned,
    AddrNotValid
}

impl KPageAllocator {
    pub fn new(mem_start: usize, size: usize) -> Result<Self, AllocErr> {
        let page_count = size / PAGE_SIZE;
        let num_bitmaps = (page_count + (BITMAP_LEN - 1)) / BITMAP_LEN;

        let map_mem = mem_start as *mut AtomicU64;
        let pmap = unsafe {
            core::slice::from_raw_parts_mut(map_mem, num_bitmaps) };

        let mut alloc_start = mem_start + (num_bitmaps * core::mem::size_of::<AtomicU64>());
        alloc_start = (alloc_start + (PAGE_SIZE - 1)) & !(PAGE_SIZE - 1);
        
        Ok(Self {
            pmap,
            alloc_start,
            page_count
        })
    }

    pub fn allocate(&mut self) -> Option<*mut u8> {
        for (idx, item) in self.pmap.iter().enumerate() {
            let map = item.load(Ordering::Relaxed);
            if map != u64::MAX {
                let bit_idx = (!map).trailing_zeros() as usize;
                if bit_idx >= BITMAP_LEN {continue;}
                let mask = 1u64 << bit_idx;
                
                item.fetch_or(mask, Ordering::SeqCst);
                
                let offset = (BITMAP_LEN * idx) + bit_idx;
                let page = (self.alloc_start + (PAGE_SIZE * offset)) as *mut u8;
                return Some(page);
            }
        }
        None
    }

    // TODO: deallocate more than one page 
    //      `pub fn deallocate(&mut self, addr: *mut u8, size: usize){` 
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

    if let Ok(allocator) = 
        &mut KPageAllocator::new(memory.as_mut_ptr() as usize, LEN){

        let allocated_arr = allocator.allocate().unwrap();
        assert!(allocator.page_allocated(allocated_arr));

        allocator.deallocate(allocated_arr);
        assert!(!allocator.page_allocated(allocated_arr))
    }
}

// https://github.com/qemu/qemu/blob/master/hw/riscv/virt.c
pub struct VirtMemMap;
impl VirtMemMap {
    pub const VIRT_DEBUG        : usize = 0x0;
    pub const VIRT_MROM         : usize = 0x1000;
    pub const VIRT_TEST         : usize = 0x100000;
    pub const VIRT_RTC          : usize = 0x101000;
    pub const VIRT_CLINT        : usize = 0x2000000;
    pub const VIRT_ACLINT_SSWI  : usize = 0x2F00000;
    pub const VIRT_PCIE_PIO     : usize = 0x3000000;
    pub const VIRT_IOMMU_SYS    : usize = 0x3010000;
    pub const VIRT_PLATFORM_BUS : usize = 0x4000000;
    pub const VIRT_PLIC         : usize = 0xc000000;
    pub const VIRT_APLIC_M      : usize = 0xc000000;
    pub const VIRT_APLIC_S      : usize = 0xd000000;
    pub const VIRT_UART0        : usize = 0x10000000;
    pub const VIRT_VIRTIO       : usize = 0x10001000;
    pub const VIRT_FW_CFG       : usize = 0x10100000;
    pub const VIRT_FLASH        : usize = 0x20000000;
    pub const VIRT_IMSIC_M      : usize = 0x24000000;
    pub const VIRT_IMSIC_S      : usize = 0x28000000;
    pub const VIRT_PCIE_ECAM    : usize = 0x30000000;
    pub const VIRT_PCIE_MMIO    : usize = 0x40000000;
    pub const VIRT_DRAM         : usize = 0x80000000;
}

pub struct PTEPerms;
impl PTEPerms {
    pub const VALID : u64 = 1u64;
    pub const READ  : u64 = 1u64 << 1;
    pub const WRITE : u64 = 1u64 << 2;
    pub const EXEC  : u64 = 1u64 << 3;
    pub const USER  : u64 = 1u64 << 4;
}


#[macro_export]
macro_rules! addr_get_page_index{
    ($addr:expr, $level:expr) => {{
        let mut index = $addr >> ( 9 * $level) + crate::virtm::PAGE_OFFSET;
        index = index & crate::virtm::LEVEL_MASK;
        index
    }};
}


/// Uses RISCV SV39 Scheme
pub fn vm_map(phys_addr: usize, vm_addr: usize, map_size: usize, perms: u64) {
    let pt_set = unsafe { KERN_SATP != 0};
    
    if pt_set {
        let pg_table_len: usize =  PAGE_SIZE / core::mem::size_of::<u64>();
        let max_addr    : usize = vm_addr + map_size;
        let mut curr_vm_addr = vm_addr;
        let mut curr_phys_addr = phys_addr;

        while curr_vm_addr < max_addr {
            let mut page_table = unsafe { KERN_SATP as *mut u64 };
            for addr_level in  (1..=2).rev(){
                let page_idx  = addr_get_page_index!(curr_vm_addr, addr_level);
                let pt_slice  = unsafe {
                    core::slice::from_raw_parts_mut(page_table, pg_table_len)
                };
                if let Some(entry) = pt_slice.get_mut(page_idx){
                    let page_valid = (*entry & PTEPerms::VALID) != 0;
                    if !page_valid { 
                        unsafe{
                            if let Some(allocator) = &mut KERN_PG_ALLOCATOR{
                                let allocated_addr = allocator.allocate();
                                match allocated_addr {
                                    Some(addr) => {
                                        let pg_index  = (addr as usize) / PAGE_SIZE;
                                        let entry_val = ((pg_index as u64) << PAGE_FLAGS) | PTEPerms::VALID | perms ;
                                        *entry = entry_val;
                                        page_table = (pg_index * PAGE_SIZE) as *mut u64;
                                        continue;
                                    },
                                    None => return
                                }
                            }
                        };
                    }
                    page_table = ((*entry >> PAGE_FLAGS) * PAGE_SIZE as u64) as *mut u64;
                };
            }

            let level_page_idx  = addr_get_page_index!(curr_vm_addr, 0);
            unsafe {
                let pt_slice = core::slice::from_raw_parts_mut(page_table, pg_table_len);
                if let Some(entry) = pt_slice.get_mut(level_page_idx){
                    let phys_pg_index = (curr_phys_addr / PAGE_SIZE) as u64;
                    *entry = (phys_pg_index << PAGE_FLAGS) | PTEPerms::VALID | perms;
                }
            };

            /* 
            TODO: We will need to be able to map contiguous VM pages
                  to non-contiguous PM pages.
            */
            curr_phys_addr += PAGE_SIZE;
            curr_vm_addr   += PAGE_SIZE;
        }
    }
}

#[inline]
fn get_end() -> usize{
    let x: usize;
    unsafe {
        core::arch::asm!(
            "la {}, end",
            out(reg) x
        )
    };
    x
}


#[inline]
fn get_txt_end() -> usize {
    let x: usize;
    unsafe {
        core::arch::asm!(
            "la {}, etext",
            out(reg) x
        )
    };
    x
}


#[inline]
fn get_kern_stack() -> usize {
    let x: usize;
    unsafe {
        core::arch::asm!(
            "la {}, stack0",
            out(reg) x
        )
    };
    x
}


pub fn kern_vm_create_maps(){
    let kern_txt_end = get_txt_end();
    let kern_stack = get_kern_stack();
    let ncpus = 2_usize;
    let stack_size = ncpus * (1024 * 4);

    vm_map(VirtMemMap::VIRT_UART0, 
           VirtMemMap::VIRT_UART0, PAGE_SIZE,
           PTEPerms::WRITE | PTEPerms::READ);
    
    vm_map(VirtMemMap::VIRT_VIRTIO, 
            VirtMemMap::VIRT_VIRTIO, PAGE_SIZE, 
            PTEPerms::WRITE | PTEPerms::READ);
    
    vm_map(VirtMemMap::VIRT_PLIC, 
            VirtMemMap::VIRT_PLIC, 0x4000000, 
            PTEPerms::WRITE | PTEPerms::READ);

    vm_map(KERN_START, KERN_START, 
            kern_txt_end - KERN_START, PTEPerms::READ | PTEPerms::EXEC);

    vm_map(kern_stack, kern_stack, stack_size, 
            PTEPerms::READ | PTEPerms::WRITE);

}



pub fn kern_vm_init(){
    let mut satp_created = false;
    unsafe {
        let kern_end = get_end();
        let mem_size = KERN_RESERV - (kern_end - KERN_START);
        if let Ok(mut kallocator) = KPageAllocator::new(kern_end, mem_size){
            {
                let alloc_ref = &mut kallocator;
                let page = alloc_ref.allocate();
                match page {
                    Some(page) => {
                        KERN_PG_ALLOCATOR = Some(kallocator);
                        KERN_SATP = page as usize;
                        satp_created = true;
                    },
                    None => {}
                }
            }
        }
    }
    if satp_created {kern_vm_create_maps();}
}
