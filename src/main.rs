#![windows_subsystem = "windows"]

use windows::core::PCSTR;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_ICONINFORMATION, MB_OK};

fn main() {
    let text = String::from("My text\0");
    let caption = String::from("My caption\0");

    unsafe {
        MessageBoxA(None, PCSTR(text.as_ptr()), PCSTR(caption.as_ptr()), MB_OK | MB_ICONINFORMATION);
    }
}