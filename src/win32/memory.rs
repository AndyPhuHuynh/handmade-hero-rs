use std::ffi::c_void;
use std::ptr::NonNull;
use windows::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE,
};

pub struct VirtualAllocMemory {
    ptr: NonNull<c_void>,
    size: usize,
}

impl VirtualAllocMemory {
    pub fn new(buffer_size: usize) -> Option<Self> {
        unsafe {
            let ptr = VirtualAlloc(None, buffer_size, MEM_COMMIT, PAGE_READWRITE);
            let ptr = NonNull::new(ptr)?;
            Some(Self {
                ptr,
                size: buffer_size,
            })
        }
    }

    pub fn data(&self) -> NonNull<c_void> {
        self.ptr
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr() as *mut u8, self.size) }
    }
}

impl Drop for VirtualAllocMemory {
    fn drop(&mut self) {
        unsafe {
            let result = VirtualFree(self.ptr.as_ptr(), 0, MEM_RELEASE);
            if let Err(e) = result {
                debug_assert!(false, "VirtualFree failed: {}", e);
            }
        }
    }
}
