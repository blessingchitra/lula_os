use crate::virtm;

pub static mut USR_PROG_START: usize = 0;
pub const USR_PROG_SIZE : usize = 12;

pub static USR_PROG: [u8; 12] = [
    0x13, 0x05, 0x20, 0x00,     // li a0, 2
    0x73, 0x00, 0x00, 0x00,     // ecall
    0x6f, 0x00, 0x00, 0x00      // j loop
];

#[unsafe(no_mangle)]
pub fn usr_mem_setup() {
    unsafe {
        if let Some(allocator) = &mut virtm::KERN_PG_ALLOCATOR {
            if let Some(page) = allocator.allocate(){
                USR_PROG_START = page as usize;
                let dst = USR_PROG_START;
                virtm::vm_map(dst, dst, 
                        USR_PROG_SIZE, virtm::PTEPerms::READ | virtm::PTEPerms::EXEC | virtm::PTEPerms::WRITE, "Custom map\n");
            }
        }
    };
}

#[unsafe(no_mangle)]
pub fn usr_load_and_exec(){
    usr_mem_setup();
    let dst  = unsafe{ USR_PROG_START };
    let src = USR_PROG.as_ptr();
    if dst == 0 {
        kprintln!("Page Not Allocated for USR program. Addr: {:#x}", dst);
        return;
    }

    let table = unsafe {virtm::KERN_SATP as *mut u64};
    virtm::addr_dbg(dst, table);

    // kprintln!("USR Prog Addr: {:#x}", dst);
    virtm::memcpy(dst as *mut u8, src, USR_PROG_SIZE);

    let prog_slice = unsafe {
        core::slice::from_raw_parts(dst as *const u8, USR_PROG_SIZE)
    };

    for (index, byte )in prog_slice.iter().enumerate() {
        let endline = if index != 0 && (index % 4) == 0 { "\n"} else {" "};
        kprint!("{:#x}{}", byte, endline);
    }
    
    unsafe {
        core::arch::asm!(
            "jr {}",
            in(reg) USR_PROG_START
        );
    };
}

