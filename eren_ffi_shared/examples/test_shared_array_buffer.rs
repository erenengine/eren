#[unsafe(no_mangle)]
pub extern "C" fn alloc_buffer(size: usize) -> *mut u8 {
    let mut buffer = Vec::with_capacity(size);
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer); // 메모리를 Rust가 회수하지 않게 함
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn free_buffer(ptr: *mut u8, size: usize) {
    unsafe {
        let buffer = Vec::from_raw_parts(ptr, size, size);
        log::debug!("Data: {:?}", buffer);
        // drop happens here
    }
}

pub fn init_logger() {
    #[cfg(target_arch = "wasm32")]
    {
        use log::Level;

        console_error_panic_hook::set_once();
        console_log::init_with_level(Level::Debug).expect("Failed to init console_log");
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
}

fn run() {
    init_logger();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
fn start() {
    run();
}

pub fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        run();
    }
}
