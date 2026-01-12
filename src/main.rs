#![windows_subsystem = "windows"]

use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, GetDC, ReleaseDC, StretchDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    DIB_RGB_COLORS, HDC, PAINTSTRUCT, SRCCOPY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, MessageBoxW, PeekMessageW,
    RegisterClassW, TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, MB_ICONINFORMATION,
    MB_OK, MSG, PM_REMOVE, WINDOW_EX_STYLE, WM_ACTIVATEAPP, WM_CLOSE, WM_DESTROY, WM_PAINT,
    WM_QUIT, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

// TODO: This is a global (EP2 15:40)
static IS_RUNNING: AtomicBool = AtomicBool::new(false);
static mut GLOBAL_BACK_BUFFER: Option<OffscreenBuffer> = None;

struct VirtualAllocMemory {
    ptr: NonNull<c_void>,
    size: usize,
}

impl VirtualAllocMemory {
    fn new(buffer_size: usize) -> Option<Self> {
        unsafe {
            let ptr = VirtualAlloc(None, buffer_size, MEM_COMMIT, PAGE_READWRITE);
            let ptr = NonNull::new(ptr)?;
            Some(Self {
                ptr,
                size: buffer_size,
            })
        }
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
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

struct OffscreenBuffer {
    info: BITMAPINFO,
    memory: VirtualAllocMemory,
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

fn popup_error(text: PCWSTR) {
    unsafe {
        MessageBoxW(
            None,
            text,
            w!("Critical Error!"),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

fn get_rect_dimensions(rect: &RECT) -> (i32, i32) {
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    (width, height)
}

fn get_client_rect_dimensions(window: HWND) -> (i32, i32) {
    let mut client_rect = RECT::default();
    unsafe {
        let _ = GetClientRect(window, &mut client_rect);
    }
    let (width, height) = get_rect_dimensions(&client_rect);
    (width, height)
}

fn render_weird_gradient(buffer: &mut OffscreenBuffer, x_offset: i32, y_offset: i32) {
    let width = buffer.width();
    let height = buffer.height();
    let pitch = buffer.pitch();
    let mem = buffer.memory.as_mut_slice();

    let mut row_index = 0;
    for y in 0..height {
        let mut byte_index = row_index;
        for x in 0..width {
            mem[byte_index] = (x + x_offset) as u8;
            byte_index += 1;

            mem[byte_index] = (y + y_offset) as u8;
            byte_index += 1;

            mem[byte_index] = (x * x + y * y + x_offset + y_offset) as u8;
            byte_index += 1;

            mem[byte_index] = 0;
            byte_index += 1;
        }
        row_index += pitch;
    }
}

fn display_buffer_in_window(
    buffer: &OffscreenBuffer,
    device_context: HDC,
    window_width: i32,
    window_height: i32,
) {
    unsafe {
        StretchDIBits(
            device_context,
            0,
            0,
            window_width,
            window_height,
            0,
            0,
            buffer.width(),
            buffer.height(),
            Some(buffer.memory.ptr.as_ptr()),
            &buffer.info,
            DIB_RGB_COLORS,
            SRCCOPY,
        );
    }
}

extern "system" fn wnd_proc(
    window: HWND,
    message: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    let mut result: LRESULT = LRESULT(0);
    match message {
        WM_CLOSE => {
            // TODO: Handle with a message (EP2 15:40)
            IS_RUNNING.store(false, Ordering::Relaxed);
        }
        WM_ACTIVATEAPP => {},
        WM_DESTROY => {
            // TODO: Handle this as error and recreate window (EP2 15:40)
            IS_RUNNING.store(false, Ordering::Relaxed);
        }
        WM_PAINT => unsafe {
            if let Some(buffer) = GLOBAL_BACK_BUFFER.as_ref() {
                let mut paint = PAINTSTRUCT::default();
                let device_context: HDC = BeginPaint(window, &mut paint);
                let (width, height) = get_client_rect_dimensions(window);
                display_buffer_in_window(&buffer, device_context, width, height);
                let _ = EndPaint(window, &mut paint);
            };
        },
        _ => result = unsafe { DefWindowProcW(window, message, w_param, l_param) },
    }
    result
}

fn main() {
    unsafe {
        let h_instance: HINSTANCE = GetModuleHandleW(PCWSTR::null())
            .expect("Unable to get hInstance")
            .into();
        let class_name = w!("HandmadeHeroWindowClass");

        let wnd_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            hInstance: h_instance,
            lpszClassName: class_name,
            ..Default::default()
        };

        if RegisterClassW(&wnd_class) == 0 {
            popup_error(w!("Failed to register window class"));
            return;
        }

        let window: HWND = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("Handmade Hero"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            Some(h_instance),
            None,
        )
        .expect("Error creating window");

        let (width, height) = get_client_rect_dimensions(window);
        GLOBAL_BACK_BUFFER = Some(OffscreenBuffer::new(width, height).expect("Unable to allocate buffer"));

        IS_RUNNING.store(true, Ordering::Relaxed);
        let mut x_offset = 0;
        let mut y_offset = 0;
        while IS_RUNNING.load(Ordering::Relaxed) {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).0 > 0 {
                if msg.message == WM_QUIT {
                    IS_RUNNING.store(false, Ordering::Relaxed);
                }
                let _ = TranslateMessage(&mut msg);
                DispatchMessageW(&mut msg);
            }

            if let Some(buffer) = GLOBAL_BACK_BUFFER.as_mut() {
                render_weird_gradient(buffer, x_offset, y_offset);
                x_offset += 1;
                y_offset += 1;

                let device_context: HDC = GetDC(Some(window));
                let (width, height) = get_client_rect_dimensions(window);

                display_buffer_in_window(buffer, device_context, width, height);
                ReleaseDC(Some(window), device_context);
            }
        }
    }
}
