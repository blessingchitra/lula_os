// Machine Status Register, mstatus
pub const MSTATUS_MPP_MASK  : usize = 3 << 11;          // previous mode.
pub const MSTATUS_MPP_M     : usize = 3 << 11;
pub const MSTATUS_MPP_S     : usize = 1 << 11;
pub const MSTATUS_MPP_U     : usize = 0 << 11;
pub const MSTATUS_MIE       : usize = 1 << 3;           // machine-mode interrupt enable.
// Machine Interrupt Enable
pub const MIE_STIE          : u64 = 1 << 5;             // supervisor timer


// Supervisor Status Register, sstatus
pub const SSTATUS_SPP       : usize = 1 << 8;           // Previous mode, 1=Supervisor, 0=User
pub const SSTATUS_SPIE      : usize = 1 << 5;           // Supervisor Previous Interrupt Enable
pub const SSTATUS_UPIE      : usize = 1 << 4;           // User Previous Interrupt Enable
pub const SSTATUS_SIE       : usize = 1 << 1;           // Supervisor Interrupt Enable
pub const SSTATUS_UIE       : usize = 1 << 0;           // User Interrupt Enable
// Supervisor Interrupt Enable
pub const SIE_SEIE          : usize = 1 << 9;           // external
pub const SIE_STIE          : usize = 1 << 5;           // timer
pub const SIE_SSIE          : usize = 1 << 1;           // software


// use riscv's sv39 page table scheme.
pub const SATP_SV39         : usize =  8 << 60;

#[macro_export]
macro_rules! make_satp{
    ($pagetable:expr) => {
        $crate::riscv::SATP_SV39 | $pagetable as u64 >> 12
    };
}

/// Return the Id of the CPU execting the code. 
/// * `u64` - The CPU Id
pub fn r_mhartid() -> u64
{
    let x: u64;
    unsafe {
        core::arch::asm!(
            "csrr {}, mhartid",
            out(reg) x
        )
    };
    x
}

// -- machine

#[inline]
pub fn r_mstatus() -> u64
{
    let x: u64;
    unsafe {
        core::arch::asm!(
            "csrr {}, mstatus",
            out(reg) x,
        )
    };
    x
}

#[inline]
pub fn w_mstatus(x: u64)
{
    unsafe {
        core::arch::asm!(
            "csrw mstatus, {}",
            in(reg) x
        )
    };
}

/// machine exception program counter
/// the pc is set to the value in this register
/// when returning from an exception
#[inline]
pub fn w_mepc(x: u64)
{
    unsafe {
        core::arch::asm!(
            "csrw mepc, {}",
            in(reg) x
        )
    }
}

// -- supervisor

#[inline]
pub fn r_sstatus() -> u64
{
    let x: u64;
    unsafe {
        core::arch::asm!(
            "csrr {}, sstatus",
            out(reg) x,
        )
    };
    x
}

#[inline]
pub fn w_sstatus(x: u64)
{
    unsafe {
        core::arch::asm!(
            "csrw sstatus, {}",
            in(reg) x
        )
    };
}

/// Supevisor Interrupt Pending
#[inline]
pub fn r_sip() -> u64
{
    let x: u64;
    unsafe {
        core::arch::asm!(
            "csrr {}, sip",
            out(reg) x,
        )
    };
    x
}

#[inline]
pub fn w_sip(x: u64)
{
    unsafe {
        core::arch::asm!(
            "csrw sip, {}",
            in(reg) x,
        )
    };
}

#[inline]
pub fn r_sie() -> u64
{
    let x: u64;
    unsafe {
        core::arch::asm!(
            "csrr {}, sie",
            out(reg) x,
        )
    };
    x
}

#[inline]
pub fn w_sie(x: u64)
{
    unsafe {
        core::arch::asm!(
            "csrw sie, {}",
            in(reg) x,
        )
    };
}


#[inline]
pub fn r_mie() -> u64
{
    let x: u64;
    unsafe {
        core::arch::asm!(
            "csrr {}, mie",
            out(reg) x,
        )
    };
    x
}

#[inline]
pub fn w_mie(x: u64)
{
    unsafe {
        core::arch::asm!(
            "csrw mie, {}",
            in(reg) x,
        )
    };
}


/// supervisor exception program counter
/// the pc is set to the value in this register
/// when returning from an exception
#[inline]
pub fn w_sepc(x: u64)
{
    unsafe {
        core::arch::asm!(
            "csrw sepc, {}",
            in(reg) x
        )
    }
}

#[inline]
pub fn r_sepc() -> u64
{
    let x: u64;
    unsafe {
        core::arch::asm!(
            "csrr {}, sepc",
            out(reg) x,
        )
    };
    x
}


/// Machine Exception Delegation
#[inline]
pub fn r_medeleg() -> u64
{
    let x: u64;
    unsafe {
        core::arch::asm!(
            "csrr {}, medeleg",
            out(reg) x,
        )
    };
    x
}

#[inline]
pub fn w_medeleg(x: u64)
{
    unsafe {
        core::arch::asm!(
            "csrw medeleg, {}",
            in(reg) x
        )
    }
}

/// Machine Interrupt Delegation
#[inline]
pub fn r_mideleg() -> u64
{
    let x: u64;
    unsafe {
        core::arch::asm!(
            "csrr {}, mideleg",
            out(reg) x,
        )
    };
    x
}

#[inline]
pub fn w_mideleg(x: u64)
{
    unsafe {
        core::arch::asm!(
            "csrw mideleg, {}",
            in(reg) x
        )
    }
}

/// Supervisor Trap Vector Address.
/// The lower 2 bit determine the mode.
/// The pc register is set to the value inside the
/// `stvec` register when an exception or interrupt happens.
#[inline]
pub fn r_stvec() -> u64
{
    let x: u64;
    unsafe {
        core::arch::asm!(
            "csrr {}, stvec",
            out(reg) x,
        )
    };
    x
}

#[inline]
pub fn w_stvec(x: u64)
{
    unsafe {
        core::arch::asm!(
            "csrw stvec, {}",
            in(reg) x
        )
    }
}

/// Supervisor Time Comparison Register
#[inline]
pub fn r_stimecmp() -> u64
{
    let x: u64;
    unsafe {
        // "csrr {}, stimecmp",
        core::arch::asm!(
            "csrr {}, 0x14d",
            out(reg) x,
        )
    };
    x
}

#[inline]
pub fn w_stimecmp(x: u64)
{
    unsafe {
        // "csrr 0x14d, {}",
        core::arch::asm!(
            "csrw 0x14d, {}",
            in(reg) x
        )
    };
}

/// Machine Environment Configuration Register
#[inline]
pub fn r_menvcfg() -> u64 
{
    let x: u64;
    unsafe {
        // "csrr {}, menvcfg",
        core::arch::asm!(
            "csrr {}, 0x30a",
            out(reg) x
        )
    };
    x
}

#[inline]
pub fn w_menvcfg(x: u64)
{
    unsafe {
        // "csrw menvcfg, {}",
        core::arch::asm!(
            "csrw 0x30a, {}",
            in(reg) x
        )
    };
}

/// Physical Memory Protection
pub fn w_pmpcfg0(x: u64)
{
    unsafe {
        core::arch::asm!(
            "csrw pmpcfg0, {}",
            in(reg) x
        )
    }
}

#[inline]
pub fn w_pmpaddr0(x: u64)
{
    unsafe {
        core::arch::asm!(
            "csrw pmpaddr0, {}",
            in(reg) x
        )
    }
}

/// Supervisor Address Translation and Protection
/// The address of the page table.
#[inline]
pub fn r_satp() -> u64
{
    let x: u64;
    unsafe {
        core::arch::asm!(
            "csrr {}, satp",
            out(reg) x
        )
    };
    x
}

#[inline]
pub fn w_satp(x: u64)
{
    unsafe {
        core::arch::asm!(
            "csrw satp, {}",
            in(reg) x
        )
    }
}


/// Supervisor trap cause
#[inline]
pub fn r_scause() -> u64
{
    let x: u64;
    unsafe {
        core::arch::asm!(
            "csrr {}, scause",
            out(reg) x
        )
    }
    x
}

/// supervisor trap value
#[inline]
pub fn r_stval() -> u64
{
    let x: u64;
    unsafe {
        core::arch::asm!(
            "csrr {}, stval",
            out(reg) x
        )
    };
    x
}

/// Machine mode counter enable
#[inline]
pub fn r_mcounteren() -> u64
{
    let x: u64;
    unsafe {
        core::arch::asm!(
            "csrr {}, mcounteren",
            out(reg) x
        )
    };
    x
}

#[inline]
pub fn w_mcounteren(x: u64)
{
    unsafe {
        core::arch::asm!(
            "csrw mcounteren, {}",
            in(reg) x
        )
    }
}

/// Machine mode cycle counter
#[inline]
pub fn r_time() -> u64
{
    let x: u64;
    unsafe {
        core::arch::asm!(
            "csrr {}, time",
            out(reg) x
        )
    };
    x
}

/// Toggle device interrupts
#[inline]
pub fn intr_on() {
    unsafe {
        core::arch::asm!("csrsi sstatus, 1 << 1"); 
    }
}

#[inline]
pub fn intr_off() {
    unsafe {
        core::arch::asm!("csrci sstatus, 1 << 1"); 
    }
}

/// Check if device interrupts are enabled
#[inline]
pub fn intr_get() -> bool
{
    let status = r_sstatus();
    let status =  (status & (SSTATUS_SIE as u64)) != 0;
    status
}

#[inline]
pub fn r_sp() -> u64
{
    let x: u64;
    unsafe {
        core::arch::asm!(
            "mv {}, sp",
            out(reg) x
        )
    }
    x
}

#[inline]
pub fn r_tp() -> u64 
{
    let x: u64;
    unsafe {
        core::arch::asm!(
            "mv {}, tp",
            out(reg) x
        )
    }
    x
}

#[inline]
pub fn w_tp(x: u64)
{
    unsafe {
        core::arch::asm!(
              "mv tp, {}",
              in(reg) x
        )
    }
}

#[inline]
pub fn r_ra() -> u64 
{
    let x: u64;
    unsafe {
        core::arch::asm!(
            "mv {}, ra",
            out(reg) x
        )
    }
    x
}





















