#![feature(lang_items)]
#![no_std]

#[no_mangle]
pub extern "C" fn rust_main() {}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
#[lang = "panic_fmt"]
extern "C" fn panic_fmt() -> ! {
    loop {}
}
