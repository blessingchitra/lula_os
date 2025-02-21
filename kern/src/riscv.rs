use crate::plic;
use crate::virtm;
use crate::uart;

#[allow(non_upper_case_globals)]
static mut sys_initialised: bool = false;

extern "C" {
    fn kern_exec() -> !;
    fn kern_trap();
}

#[export_name = "sys_init"]
pub extern "C" fn sys_init()
{
    let mut x = RegMStatus::read();
    x &= !RegMStatus::MSTATUS_MPP_MASK;
    x |=  RegMStatus::MSTATUS_MPP_S;
    RegMStatus::write(x);

    RegMEPC::write(kern_exec as usize);

    RegSTVec::write(kern_trap as usize);

    RegMEDeleg::write(0xffff);
    RegMIDeleg::write(0xffff);

    let mut intr = RegSIE::read(); 
    intr |= RegSIE::SIE_SEIE;
    intr |= RegSIE::SIE_SSIE;
    intr |= RegSIE::SIE_STIE; 

    RegSIE::write(intr);
    intr_on();
    
    // 
    RegPmpAddr0::write(0x3fffffffffffff);
    RegPmpCfg0::write(0xf);
    let cpu_id = RegMHartId::read();
    RegTP::write(cpu_id);

    if cpu_id == 0 { 
        uart::uart_init();
        plic::plic_init(0); 
        virtm::kern_vm_init();
        unsafe {sys_initialised = true};
    }

    while (cpu_id != 0) && !unsafe { sys_initialised } { }

    unsafe {
        RegSATP::set_root_page_sv39_(virtm::KERN_SATP);
        core::arch::asm!("mret")
    };
}


/// Toggle Interrupts
#[inline]
pub fn intr_on() {
    RegSStatus::intr_on();
}

#[inline]
pub fn intr_off() {
    RegSStatus::intr_off();
}

#[inline]
pub fn intr_get() -> bool
{
    RegSStatus::intr_get()
}



pub trait Register {
    fn read() -> usize;
    fn write(_x: usize){}
}

pub struct RegMHartId;
impl Register for RegMHartId {
    fn read() -> usize {
        let x: usize;
        unsafe {
            core::arch::asm!(
                "csrr {}, mhartid",
                out(reg) x
            )
        };
        x
    }
}


pub struct RegMStatus;
impl RegMStatus {
    pub const MSTATUS_MPP_MASK: usize = 3 << 11;  // previous mode.
    pub const MSTATUS_MPP_M   : usize = 3 << 11;
    pub const MSTATUS_MPP_S   : usize = 1 << 11;
    pub const MSTATUS_MPP_U   : usize = 0 << 11;
    pub const MSTATUS_MIE     : usize = 1 << 3;   // machine-mode interrupt enable.
}
impl Register for RegMStatus{
    fn read() -> usize {
        let x: usize;
        unsafe {
            core::arch::asm!(
                "csrr {}, mstatus",
                out(reg) x,
            )
        };
        x
    }

    fn write(x: usize) {
        unsafe {
            core::arch::asm!(
                "csrw mstatus, {}",
                in(reg) x
            )
        };
    }
}


pub struct RegMEPC;
impl Register for RegMEPC{
    fn read() -> usize {0usize}

    fn write(x: usize) {
        unsafe {
            core::arch::asm!(
                "csrw mepc, {}",
                in(reg) x
            )
        }
    }
}

// -- supervisor
pub struct RegSStatus;
impl Register for RegSStatus {
    #[inline]
    fn read() -> usize {
        let x: usize;
        unsafe {
            core::arch::asm!(
                "csrr {}, sstatus",
                out(reg) x
            )
        };
        x
    }

    fn write(x: usize) {
        unsafe {
            core::arch::asm!(
                "csrw sstatus, {}",
                in(reg) x
            )
        };
    }
}

impl RegSStatus {
    pub const SSTATUS_SPP  : usize = 1 << 8;  // Previous mode, 1=Supervisor, 0=User
    pub const SSTATUS_SPIE : usize = 1 << 5;  // Supervisor Previous Interrupt Enable
    pub const SSTATUS_UPIE : usize = 1 << 4;  // User Previous Interrupt Enable
    pub const SSTATUS_SIE  : usize = 1 << 1;  // Supervisor Interrupt Enable
    pub const SSTATUS_UIE  : usize = 1 << 0;  // User Interrupt Enable

    pub const SSTATUS_MXR  : usize = 1 << 19;

    pub fn intr_on() {
        unsafe {
            core::arch::asm!("csrsi sstatus, 1 << 1"); 
        }
    }
    pub fn intr_off() {
        unsafe {
            core::arch::asm!("csrci sstatus, 1 << 1"); 
        }
    }
    pub fn intr_get() -> bool
    {
        let status = RegSStatus::read();
        let status =  (status & RegSStatus::SSTATUS_SIE) != 0;
        status
    }
}


pub struct RegSIP;
impl Register for RegSIP {
    // Supevisor Interrupt Pending
    fn read() -> usize {
        let x: usize;
        unsafe {
            core::arch::asm!(
                "csrr {}, sip",
                out(reg) x,
            )
        };
        x
    }

    fn write(x: usize) {
        unsafe {
            core::arch::asm!(
                "csrw sip, {}",
                in(reg) x,
            )
        };
    }
}


pub struct RegSIE;
impl RegSIE {
    pub const SIE_SEIE : usize = 1 << 9;  // external
    pub const SIE_STIE : usize = 1 << 5;  // timer
    pub const SIE_SSIE : usize = 1 << 1;  // software
}

impl Register for RegSIE{
    fn read() -> usize {
        let x: usize;
        unsafe {
            core::arch::asm!(
                "csrr {}, sie",
                out(reg) x,
            )
        };
        x
    }

    fn write(x: usize) {
        unsafe {
            core::arch::asm!(
                "csrw sie, {}",
                in(reg) x,
            )
        };
    }
}

pub struct RegMIE;
impl Register for RegMIE{
    fn read() -> usize {
        let x: usize;
        unsafe {
            core::arch::asm!(
                "csrr {}, mie",
                out(reg) x,
            )
        };
        x
    }

    fn write(x: usize) {
        unsafe {
            core::arch::asm!(
                "csrw mie, {}",
                in(reg) x,
            )
        };
    }
}

pub struct RegSEPC;
impl Register for RegSEPC{
    fn read() -> usize {
        let x: usize;
        unsafe {
            core::arch::asm!(
                "csrr {}, sepc",
                out(reg) x,
            )
        };
        x
    }

    fn write(x: usize) {
        unsafe {
            core::arch::asm!(
                "csrw sepc, {}",
                in(reg) x
            )
        }
    }
}


/// Machine Exception Delegation
pub struct RegMEDeleg;
impl Register for RegMEDeleg{
    fn read() -> usize {
        let x: usize;
        unsafe {
            core::arch::asm!(
                "csrr {}, medeleg",
                out(reg) x,
            )
        };
        x
    }
    fn write(x: usize) {
        unsafe {
            core::arch::asm!(
                "csrw medeleg, {}",
                in(reg) x
            )
        }
    }
    
}


/// Machine Interrupt Delegation
pub struct RegMIDeleg;
impl Register for RegMIDeleg {
    fn read() -> usize {
        let x: usize;
        unsafe {
            core::arch::asm!(
                "csrr {}, mideleg",
                out(reg) x,
            )
        };
        x
    }

    fn write(x: usize) {
        unsafe {
            core::arch::asm!(
                "csrw mideleg, {}",
                in(reg) x
            )
        }
    }
    
}


/// Supervisor Trap Vector Address.
/// The lower 2 bit determine the mode.
/// The pc register is set to the value inside the
/// `stvec` register when an exception or interrupt happens.
pub struct RegSTVec;
impl Register for RegSTVec {
    fn read() -> usize {
        let x: usize;
        unsafe {
            core::arch::asm!(
                "csrr {}, stvec",
                out(reg) x,
            )
        };
        x
    }

    fn write(x: usize) {
        unsafe {
            core::arch::asm!(
                "csrw stvec, {}",
                in(reg) x
            )
        }
    }
    
}

/// Supervisor Time Comparison Register
pub struct RegSTimeCmp;
impl Register for RegSTimeCmp{
    fn read() -> usize {
        let x: usize;
        unsafe {
            // "csrr {}, stimecmp",
            core::arch::asm!(
                "csrr {}, 0x14d",
                out(reg) x,
            )
        };
        x
    }

    fn write(x: usize) {
        unsafe {
            // "csrr 0x14d, {}",
            core::arch::asm!(
                "csrw 0x14d, {}",
                in(reg) x
            )
        };
    }
}


/// Machine Environment Configuration Register
pub struct RegMEnvCfg;
impl Register for RegMEnvCfg{
    fn read() -> usize {
        let x: usize;
        unsafe {
            // "csrr {}, menvcfg",
            core::arch::asm!(
                "csrr {}, 0x30a",
                out(reg) x
            )
        };
        x
    }

    fn write(x: usize) {
        unsafe {
            // "csrw menvcfg, {}",
            core::arch::asm!(
                "csrw 0x30a, {}",
                in(reg) x
            )
        };
    }
}


/// Physical Memory Protection
pub struct RegPmpCfg0;
impl Register for RegPmpCfg0 {
    fn read() -> usize { 0 }

    fn write(x: usize) {
        unsafe {
            core::arch::asm!(
                "csrw pmpcfg0, {}",
                in(reg) x
            )
        }
    }
}


pub struct RegPmpAddr0;
impl Register for RegPmpAddr0 {
    fn read() -> usize { 0 }

    fn write(x: usize) {
        unsafe {
            core::arch::asm!(
                "csrw pmpaddr0, {}",
                in(reg) x
            )
        }
    }
    
}


#[export_name = "prop_satp"]
fn prop_satp(){
    let x = 10;
    let z = x + 3;
}

/// The address of the page table.
pub struct RegSATP;
impl RegSATP {
    pub const SV39: u64 = 8 << 60;
    pub fn read() -> usize {
        let x: usize;
        unsafe {
            core::arch::asm!(
                "csrr {}, satp",
                out(reg) x
            )
        };
        x
    }

    pub fn write(x: u64) {
        unsafe {
            core::arch::asm!(
                "csrw satp, {}",
                in(reg) x
            )
        }
    }

    fn set_root_page_sv39_(addr: u64){
        unsafe { core::arch::asm!("sfence.vma zero, zero"); };
        let addr = (addr >> 12) | RegSATP::SV39;
        RegSATP::write(addr);
        unsafe { core::arch::asm!("sfence.vma zero, zero"); };
    }

}

/// Supervisor trap cause
pub struct RegSCause;
impl Register for RegSCause {
    fn read() -> usize {
        let x: usize;
        unsafe {
            core::arch::asm!(
                "csrr {}, scause",
                out(reg) x
            )
        }
        x
    }
    
}

/// supervisor trap value
pub struct RegSTVal;
impl Register for RegSTVal {
    fn read() -> usize {
        let x: usize;
        unsafe {
            core::arch::asm!(
                "csrr {}, stval",
                out(reg) x
            )
        };
        x
    }
}

/// Machine mode counter enable
pub struct RegMCounterEn;
impl Register for RegMCounterEn {
    fn read() -> usize {
       let x: usize;
        unsafe {
            core::arch::asm!(
                "csrr {}, mcounteren",
                out(reg) x
            )
        };
        x
    }

    fn write(x: usize) {
        unsafe {
            core::arch::asm!(
                "csrw mcounteren, {}",
                in(reg) x
            )
        }
    }
    
}

/// Machine mode cycle counter
pub struct RegTime;
impl Register for RegTime {
    fn read() -> usize {
        let x: usize;
        unsafe {
            core::arch::asm!(
                "csrr {}, time",
                out(reg) x
            )
        };
        x
    }
    
}


pub struct RegSP;
impl Register for RegSP{
    fn read() -> usize {
        let x: usize;
        unsafe {
            core::arch::asm!(
                "mv {}, sp",
                out(reg) x
            )
        }
        x
    }
}

pub struct RegTP;
impl Register for RegTP{
    fn read() -> usize {
        let x: usize;
        unsafe {
            core::arch::asm!(
                "mv {}, tp",
                out(reg) x
            )
        }
        x
    }

    fn write(x: usize) {
        unsafe {
            core::arch::asm!(
                "mv tp, {}",
                in(reg) x
            )
        }
    }
}

pub struct RegRA;
impl Register for RegRA {
    fn read() -> usize {
        let x: usize;
        unsafe {
            core::arch::asm!(
                "mv {}, ra",
                out(reg) x

            )
        }
        x
    }
}





