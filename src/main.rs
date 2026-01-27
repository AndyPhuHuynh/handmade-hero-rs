#![windows_subsystem = "windows"]

mod win32;

use std::marker::PhantomPinned;
use std::pin::Pin;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, RECT, };
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, GetDC, ReleaseDC, StretchDIBits, DIB_RGB_COLORS, HDC, PAINTSTRUCT, SRCCOPY};
use windows::Win32::UI::WindowsAndMessaging::{DispatchMessageW, GetClientRect, MessageBoxW, PeekMessageW, TranslateMessage, MB_ICONINFORMATION, MB_OK, MSG, PM_REMOVE, WM_QUIT};

use win32::buffer::OffscreenBuffer;
use crate::win32::window::create_window;


fn popup_error(text: &str) {
    let wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
    let ptr = PCWSTR(wide.as_ptr());

    unsafe {
        MessageBoxW(
            None,
            ptr,
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
            Some(buffer.memory.data().as_ptr()),
            &buffer.info,
            DIB_RGB_COLORS,
            SRCCOPY,
        );
    }
}

pub struct App {
    pub window: HWND,
    is_running: bool,
    pub back_buffer: OffscreenBuffer,
    _pin: PhantomPinned,
}

impl App {
    pub fn get_running(&self) -> bool {
        self.is_running
    }

    pub fn stop(&mut self) {
        self.is_running = false;
    }

    pub fn paint(&mut self) {
        let mut paint = PAINTSTRUCT::default();
        let device_context: HDC = unsafe { BeginPaint(self.window, &mut paint) };
        let (width, height) = get_client_rect_dimensions(self.window);
        display_buffer_in_window(&self.back_buffer, device_context, width, height);
        let _ = unsafe { EndPaint(self.window, &mut paint) };
    }
}

impl App {
    pub fn new() -> Pin<Box<Self>> {
        let (width, height) = (1280, 720);
        let buffer = OffscreenBuffer::new(width, height).expect("Unable to allocate buffer");

        let mut app = Box::pin(Self {
            window: HWND::default(),
            is_running: true,
            back_buffer: buffer,
            _pin: PhantomPinned,
        });

        let window = match create_window(width, height, app.as_ref()) {
            Ok(window) => window,
            Err(error) => {
                popup_error(error);
                panic!("Error creating window")
            }
        };
        unsafe {
            app.as_mut().get_unchecked_mut().window = window;
        }

        app
    }

    pub fn run(&mut self) {
        self.is_running = true;
        while self.is_running {
            unsafe {
                let mut x_offset = 0;
                let mut y_offset = 0;
                while self.is_running {
                    let mut msg = MSG::default();
                    while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).0 > 0 {
                        if msg.message == WM_QUIT {
                            self.stop();
                        }
                        let _ = TranslateMessage(&mut msg);
                        DispatchMessageW(&mut msg);
                    }

                    render_weird_gradient(&mut self.back_buffer, x_offset, y_offset);
                    x_offset += 1;
                    y_offset += 1;

                    let device_context: HDC = GetDC(Some(self.window));
                    let (width, height) = get_client_rect_dimensions(self.window);

                    display_buffer_in_window(&mut self.back_buffer, device_context, width, height);
                    ReleaseDC(Some(self.window), device_context);
                }
            }
        }
    }
}

fn main() {
    let mut app = App::new();
    unsafe {
        app.as_mut().get_unchecked_mut().run();
    }
}
