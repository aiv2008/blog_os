#![no_std]
#![no_main]
use core::panic::PanicInfo;
mod vga_buffer;
static HELLO: &[u8] = b"hello world";

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // let vga_buffer = 0xb8000 as *mut u8;
    // for (i, &byte) in HELLO.iter().enumerate() {
    //     unsafe {
    //         *vga_buffer.offset(i as isize * 2) = byte;
    //         *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
    //     }
    // }
    // vga_buffer::Writer::print_sth();
    use core::fmt::Write;
    vga_buffer::WRITER.lock().write_byte(b'H');
    vga_buffer::WRITER.lock().write_str("ello world\n").unwrap();
    write!(
        vga_buffer::WRITER.lock(),
        "the number are {} and {}",
        34,
        1.0 / 3.2
    )
    .unwrap();
    loop {}
    // unimplemented!()
}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
