#![no_std]
#![no_main]
#[unsafe(no_mangle)]
extern "C" fn _start() -> !{
    loop{}
}

use core::panic::PanicInfo ;

#[panic_handler]
fn panic(_info: &PanicInfo ) -> !{
    loop {
            
    }
}