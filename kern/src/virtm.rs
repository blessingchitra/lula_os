use crate::sync::SpinLock;

static kern_pages = SpinLock::new(KPage)

static MEM_CONF: MemConf = MemConf {
            kern_start:  0x80000000, 
            kern_rserv:  0x80000000 + (128 * 1024 * 1024), 
            max_addr:    1 << (9 + 9 + 9 + 12),
            page_size:   4096,
        };

pub struct Page {
    next: Option<*mut Page>,
}

impl Page {
    pub fn new() -> Self {
        Self{
            next: None
        }
    }

    pub fn set_next(&mut self, next: *mut Page){
        self.next = Some(next)
    }

    pub fn get_next(&self) -> Option<*mut Page>{
        self.next
    }
}


pub struct KPage {
    pages: Page
}


pub fn addr_page_aligned(addr: usize) -> bool {
    (addr % MEM_CONF.page_size) == 0
}

impl KPage {
    pub fn pages_from_kern_end(&mut self){
        let addr = get_kern_end();
        if addr_page_aligned(addr){
            loop {
                let valid_size = (addr + MEM_CONF.page_size) < MEM_CONF.kern_rserv;
                if !valid_size { break }
                match self.pages {
                    Some(page) => {
                        let top = self.pages
                    },
                    None =>  self.pages = 
                }
            }
        }
    }
}


pub struct MemConf{
    kern_start: usize, // text start
    kern_rserv: usize, // total reserved
    max_addr:   usize,
    page_size:  usize,
}

unsafe impl Send for MemConf {}
unsafe impl Sync for MemConf {}

extern "C" {
    static end: usize;
}


pub fn get_kern_end() -> usize {
    unsafe {end}
}










































