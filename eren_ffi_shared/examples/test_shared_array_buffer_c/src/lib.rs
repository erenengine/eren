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