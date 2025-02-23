use core::sync::atomic::{AtomicU64, Ordering};
use crate::uart;

const PAGE_SIZE   : usize = 4096;
const BITMAP_LEN  : usize = 64;
const PAGE_OFFSET : usize = 12;
const PAGE_FLAGS  : u8    = 10;
const LEVEL_MASK  : usize = 0x1FF;


pub const KERN_START  : usize = 0x80000000;
pub const KERN_RESERV : usize = 128 * (1024 * 1024);
pub const MEM_MAX : usize = 1usize << (9 + 9 + 9 + 12 - 1);

pub static mut KERN_SATP: u64 = 0;
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
                // unsafe { core::ptr::write_bytes(page, 0, PAGE_SIZE); };
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

#[export_name = "vm_map_exit"]
fn vm_map_exit(pages: i32, arr_idx: usize){
    let s = 3;
    let _m = s + 2;
}

// debug-staff
static mut addr_entries: [(usize, usize, usize, usize); 6] = [(0, 0, 0, 0),(0, 0, 0, 0),(0, 0, 0, 0),(0, 0, 0, 0),(0, 0, 0, 0),(0, 0, 0, 0),];

/// Uses RISCV SV39 Scheme
#[unsafe(no_mangle)]
pub fn vm_map(phys_addr: usize, vm_addr: usize, map_size: usize, perms: u64, region: &str) {
    uart::uart_puts(region);

    let kern_end = get_end();
    if (phys_addr % PAGE_SIZE) != 0 || (vm_addr % PAGE_SIZE) != 0 {
        kprintln!("Cannot map address. Not Aligned.{:#x} {:#x} {}", phys_addr, kern_end, region);
        return;
    }
    let pt_set = unsafe { KERN_SATP != 0};
    let mut num_pages = 0;
    let mut arr_idx = 0;
    
    if pt_set {
        let pg_table_len: usize =  PAGE_SIZE / core::mem::size_of::<u64>();
        let max_addr    : usize = vm_addr + map_size;
        let mut curr_vm_addr = vm_addr;
        let mut curr_phys_addr = phys_addr;

        while curr_vm_addr < max_addr {
            num_pages += 1;
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
                                        // core::ptr::write_bytes(addr, 0, PAGE_SIZE);
                                        let pg_index  = (addr as usize) / PAGE_SIZE;
                                        let entry_val = ((pg_index as u64) << PAGE_FLAGS) | PTEPerms::VALID ;
                                        *entry = entry_val;
                                        page_table = (pg_index * PAGE_SIZE) as *mut u64;

                                        // debug staff
                                        if arr_idx < 6 {
                                            addr_entries[arr_idx] = (curr_phys_addr, curr_vm_addr, page_idx, addr_level);
                                            arr_idx += 1;
                                        }
                                        continue;
                                    },
                                    None => {
                                        uart::uart_puts(" Could not allocate page. region: ");
                                        uart::uart_puts(region);
                                        return;
                                    }
                                }
                            }
                        };
                    }
                    page_table = ((*entry >> PAGE_FLAGS) * PAGE_SIZE as u64) as *mut u64;
                }else{
                    kprintln!("Invalid index into page table");
                    return;
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
        vm_map_exit(num_pages, arr_idx);
    }
}


#[allow(unused)]
pub struct AddrDebug {
    address     : usize,
    is_valid    : bool,
    is_writable : bool,
    is_exec     : bool,
    is_read     : bool,

}
impl core::fmt::Debug for AddrDebug {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AddrDebug")
            .field("address", &format_args!("{:#x}", self.address))
            .field("is_valid", &self.is_valid)
            .field("is_writable", &self.is_writable)
            .field("is_exec", &self.is_exec)
            .field("is_read", &self.is_read)
            .finish()
    }
}

// walk RISC SV39 page table and get debug info 
#[unsafe(no_mangle)]
pub fn addr_dbg(addr: usize, page_table: *mut u64) -> AddrDebug {
    let mut table = page_table;
    let mut table_entry: u64 = 0;
    let entry_count = PAGE_SIZE / core::mem::size_of::<u64>();

    let dbg_info = AddrDebug {
        address     : addr,
        is_valid    : false,
        is_read     : false,
        is_writable : false,
        is_exec     : false,
    };

    for level in (0..=2).rev() {
        let idx = addr_get_page_index!(addr, level);
        let entries = unsafe{
            core::slice::from_raw_parts(table, entry_count) };
        match entries.get(idx) {
            Some(entry) => {
                table_entry = *entry;
                let is_valid   = (table_entry & PTEPerms::VALID) != 0;
                if !is_valid { return dbg_info; }
                let page_no = table_entry >> PAGE_FLAGS; // PAGE_FLAGS = 10
                table  = ((PAGE_SIZE as u64) * page_no) as *mut u64;
            },
            None => {return dbg_info;} 
        }
    }
    let is_valid    = (table_entry & PTEPerms::VALID) != 0;
    let is_writable = (table_entry & PTEPerms::WRITE) != 0;
    let is_exec     = (table_entry & PTEPerms::EXEC)  != 0;
    let is_read     = (table_entry & PTEPerms::READ)  != 0;
    AddrDebug {address: addr, is_valid, is_writable, is_exec, is_read }
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

pub fn get_data_end() -> usize {
    let x: usize;
    unsafe {
        core::arch::asm!(
            "la {}, end",
            out(reg) x
        );
    };
    x
}

fn get_data_start() -> usize {
    let x: usize;
    unsafe {
        core::arch::asm!(
            "la {}, data_start",
            out(reg) x
        );
    };
    x
}

#[unsafe(no_mangle)]
pub fn kern_vm_create_maps(){
    let kern_txt_end = get_txt_end();

    let kern_data_start = get_data_start();
    let kern_data_end  = get_data_end();
    let kern_data_size = kern_data_end - kern_data_start;


    let mut kern_end = get_end();
        // let mut alloc_start = mem_start + (num_bitmaps * core::mem::size_of::<AtomicU64>());
    kern_end = (kern_end + (PAGE_SIZE - 1)) & !(PAGE_SIZE - 1);
    let mem_size = KERN_RESERV - (kern_end - KERN_START);

    vm_map(kern_end, kern_end, mem_size, 
            PTEPerms::READ | PTEPerms::WRITE | PTEPerms::EXEC, "Free Range\n");

    vm_map(KERN_START, KERN_START, 
            kern_txt_end - KERN_START, PTEPerms::READ | PTEPerms::EXEC, "Kern Code\n");

    vm_map(VirtMemMap::VIRT_UART0, 
           VirtMemMap::VIRT_UART0, PAGE_SIZE,
           PTEPerms::WRITE | PTEPerms::READ, "Uart\n");
    
    vm_map(VirtMemMap::VIRT_VIRTIO, 
            VirtMemMap::VIRT_VIRTIO, PAGE_SIZE, 
            PTEPerms::WRITE | PTEPerms::READ, "Virt IO\n");

    
    vm_map(VirtMemMap::VIRT_PLIC, 
            VirtMemMap::VIRT_PLIC, 0x4000000, 
            PTEPerms::WRITE | PTEPerms::READ, "PLIC\n");

    // TODO: FIXME: This currently marks all the kernel data (`rodata`, `data`, `bss`) 
    //       with read and write perms.
    vm_map(kern_data_start, kern_data_start, kern_data_size, 
            PTEPerms::READ | PTEPerms::WRITE, "Data Section\n");

    // let dbg_info = addr_dbg(KERN_START, table as *mut u64);
    // kprintln!("kern dbg: {:?}", dbg_info);
 }


#[unsafe(no_mangle)]
pub fn kern_vm_init(){
    let mut satp_created = false;
    unsafe {
        let mut kern_end = get_end();
        kern_end = (kern_end + (PAGE_SIZE - 1)) & !(PAGE_SIZE - 1);
        let mem_size = KERN_RESERV - (kern_end - KERN_START);
        if let Ok(mut kallocator) = KPageAllocator::new(kern_end, mem_size){
            {
                let alloc_ref = &mut kallocator;
                let page = alloc_ref.allocate();
                match page {
                    Some(page) => {
                        KERN_PG_ALLOCATOR = Some(kallocator);
                        KERN_SATP = page as u64;
                        satp_created = true;
                    },
                    None => {}
                }
            }
        }
    }
    if satp_created {kern_vm_create_maps();}
}

pub extern "C" fn memcpy(dst: *mut u8, src: *const u8, count: usize) -> *mut u8{
    unsafe {
        core::ptr::copy_nonoverlapping(src, dst, count);
    };
    dst
}