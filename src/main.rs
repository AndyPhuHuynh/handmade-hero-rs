#![windows_subsystem = "windows"]

use windows::core::{w, BOOL, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PatBlt, HDC, PAINTSTRUCT, WHITENESS};
use windows::Win32::System::Diagnostics::Debug::OutputDebugStringW;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, MessageBoxW, RegisterClassW,
    TranslateMessage, CS_HREDRAW, CS_OWNDC, CS_VREDRAW, CW_USEDEFAULT, MB_ICONINFORMATION, MB_OK,
    MSG, WINDOW_EX_STYLE, WM_ACTIVATEAPP, WM_CLOSE, WM_DESTROY, WM_PAINT, WM_SIZE, WNDCLASSW,
    WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

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

extern "system" fn wnd_proc(
    window: HWND,
    message: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    let mut result: LRESULT = LRESULT(0);
    match message {
        WM_SIZE => unsafe {
            OutputDebugStringW(w!("WM_SIZE"));
        },
        WM_DESTROY => unsafe {
            OutputDebugStringW(w!("WM_DESTROY"));
        },
        WM_CLOSE => unsafe {
            OutputDebugStringW(w!("WM_CLOSE"));
        },
        WM_ACTIVATEAPP => unsafe {
            OutputDebugStringW(w!("WM_ACTIVATEAPP"));
        },
        WM_PAINT => unsafe {
            let mut paint = PAINTSTRUCT::default();
            let device_context: HDC = BeginPaint(window, &mut paint);

            let width = paint.rcPaint.right - paint.rcPaint.left;
            let height = paint.rcPaint.bottom - paint.rcPaint.top;
            let _ = PatBlt(
                device_context,
                paint.rcPaint.left,
                paint.rcPaint.top,
                width,
                height,
                WHITENESS,
            );

            let _ = EndPaint(window, &mut paint);
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

        MessageBoxW(
            None,
            w!("My text!"),
            w!("My caption!"),
            MB_OK | MB_ICONINFORMATION,
        );

        let wnd_class = WNDCLASSW {
            style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            hInstance: h_instance,
            lpszClassName: class_name,
            ..Default::default()
        };

        if RegisterClassW(&wnd_class) == 0 {
            popup_error(w!("Failed to register window class"));
            return;
        }

        let _: HWND = CreateWindowExW(
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

        let mut msg = MSG::default();
        loop {
            let msg_result: BOOL = GetMessageW(&mut msg, None, 0, 0);
            if msg_result.0 > 0 {
                let _ = TranslateMessage(&mut msg);
                DispatchMessageW(&mut msg);
            } else {
                break;
            }
        }
    }
}
