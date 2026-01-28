use crate::App;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetWindowLongPtrW, RegisterClassW, SetWindowLongPtrW,
    CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, WINDOW_EX_STYLE,
    WM_ACTIVATEAPP, WM_CLOSE, WM_CREATE, WM_DESTROY, WM_PAINT, WNDCLASSW, WS_OVERLAPPEDWINDOW,
    WS_VISIBLE,
};

pub extern "system" fn wnd_proc(
    window: HWND,
    message: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    let mut result: LRESULT = LRESULT(0);
    match message {
        WM_CREATE => unsafe {
            let create_struct = l_param.0 as *const CREATESTRUCTW;
            let app = (*create_struct).lpCreateParams as *mut App;
            SetWindowLongPtrW(window, GWLP_USERDATA, app as isize);
        },
        WM_CLOSE => unsafe {
            // TODO: Handle with a message (EP2 15:40)
            let app = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut App;
            (*app).stop();
        },
        WM_ACTIVATEAPP => {}
        WM_DESTROY => unsafe {
            // TODO: Handle this as error and recreate window (EP2 15:40)
            let app = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut App;
            (*app).stop();
        },
        WM_PAINT => unsafe {
            let app = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut App;
            (*app).paint();
        },
        _ => result = unsafe { DefWindowProcW(window, message, w_param, l_param) },
    }
    result
}

pub fn create_window(width: i32, height: i32, app: &App) -> Result<HWND, &'static str> {
    let h_instance: HINSTANCE = unsafe {
        GetModuleHandleW(PCWSTR::null())
            .expect("Unable to get hInstance")
            .into()
    };
    let class_name = w!("HandmadeHeroWindowClass");

    let wnd_class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        hInstance: h_instance,
        lpszClassName: class_name,
        ..Default::default()
    };

    if unsafe { RegisterClassW(&wnd_class) } == 0 {
        return Err("Failed to register window class");
    }

    let window: HWND = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("Handmade Hero"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            width,
            height,
            None,
            None,
            Some(h_instance),
            Some(app as *const App as *const std::ffi::c_void),
        )
        .expect("Error creating window")
    };
    Ok(window)
}
