use windows::Win32::Graphics::Gdi::{BITMAPINFO, BITMAPINFOHEADER, BI_RGB};
use crate::win32::memory::VirtualAllocMemory;

pub struct OffscreenBuffer {
    pub info: BITMAPINFO,
    pub memory: VirtualAllocMemory,
}

impl OffscreenBuffer {
    pub const BYTES_PER_PIXEL: i32 = 4;

    pub fn width(&self) -> i32 {
        self.info.bmiHeader.biWidth
    }

    pub fn height(&self) -> i32 {
        self.info.bmiHeader.biHeight.abs()
    }

    pub fn pitch(&self) -> usize {
        (self.width() * Self::BYTES_PER_PIXEL) as usize
    }
}

impl OffscreenBuffer {
    pub fn new(width: i32, height: i32) -> Result<Self, &'static str> {
        if width <= 0 {
            return Err("bitmap width must be positive");
        }
        if height <= 0 {
            return Err("bitmap height must be positive");
        }

        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let bitmap_mem_size = width * height * Self::BYTES_PER_PIXEL;
        let memory = match VirtualAllocMemory::new(bitmap_mem_size as usize) {
            None => return Err("Unable to allocate memory with VirtualAlloc"),
            Some(mem) => mem,
        };

        Ok(Self { info, memory })
    }
}
