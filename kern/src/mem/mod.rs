pub mod alloc;
pub mod virtm;


pub fn mem_init(){
    alloc::allocator_init();
    virtm::virtm_init();
}