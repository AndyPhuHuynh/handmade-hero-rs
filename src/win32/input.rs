use libloading::Library;
use std::sync::OnceLock;
use windows::Win32::UI::Input::XboxController::{XINPUT_STATE, XINPUT_VIBRATION};

type XInputGetStateFn = unsafe fn(u32, *mut XINPUT_STATE) -> u32;
type XInputSetStateFn = unsafe fn(u32, *const XINPUT_VIBRATION) -> u32;

#[derive(Default)]
struct XInputFunctions {
    get_state: OnceLock<Result<XInputGetStateFn, String>>,
    set_state: OnceLock<Result<XInputSetStateFn, String>>,
}

static XINPUT_LIB: OnceLock<Library> = OnceLock::new();
static XINPUT_FNS: OnceLock<XInputFunctions> = OnceLock::new();

fn load_xinput_dll() -> Result<(), String> {
    unsafe {
        let lib = Library::new("xinput1_4.dll")
            .or_else(|_| Library::new("xinput1_3.dll"))
            .or_else(|_| Library::new("xinput1_2.dll"))
            .or_else(|_| Library::new("xinput1_1.dll"))
            .or_else(|_| Library::new("xinput9_1_0.dll"))
            .map_err(|e| format!("Failed to load XInput dll: {}", e))?;
        let _ = XINPUT_LIB.set(lib);
    }
    Ok(())
}

pub fn load_xinput_get_state() -> Result<XInputGetStateFn, String> {
    let lib = get_xinput_dll()?;
    match unsafe { lib.get(b"XInputGetState") } {
        Ok(s) => Ok(*s),
        Err(err) => Err(format!("Failed to load XInputGetState: {}", err).into()),
    }
}

pub fn load_xinput_set_state() -> Result<XInputSetStateFn, String> {
    let lib = get_xinput_dll()?;
    match unsafe { lib.get(b"XInputSetState") } {
        Ok(s) => Ok(*s),
        Err(err) => Err(format!("Failed to load XInputSetState: {}", err).into()),
    }
}

pub fn load_xinput() -> Result<(), String> {
    if let (Some(_), Some(_)) = (XINPUT_LIB.get(), XINPUT_FNS.get()) {
        return Ok(());
    }
    load_xinput_dll()?;
    let _ = XINPUT_FNS.set(XInputFunctions {
        get_state: load_xinput_get_state().into(),
        set_state: load_xinput_set_state().into(),
    });
    Ok(())
}

fn get_xinput_dll() -> Result<&'static Library, String> {
    load_xinput_dll()?;
    match XINPUT_LIB.get() {
        Some(lib) => Ok(lib),
        None => Err("Failed to load XInput dll".into()),
    }
}

fn get_xinput_functions() -> &'static XInputFunctions {
    XINPUT_FNS.get_or_init(|| XInputFunctions::default())
}

pub fn xinput_get_state(user_index: u32, state: *mut XINPUT_STATE) -> Result<u32, String> {
    let fns = get_xinput_functions();
    let get_state = fns.get_state.get_or_init(load_xinput_get_state);
    match get_state {
        Ok(func) => unsafe { Ok(func(user_index, state)) },
        Err(err) => Err(err.clone()),
    }
}

pub fn xinput_set_state(
    user_index: u32,
    vibration: *const XINPUT_VIBRATION,
) -> Result<u32, String> {
    let fns = get_xinput_functions();
    let set_state = fns.set_state.get_or_init(load_xinput_set_state);
    match set_state {
        Ok(func) => unsafe { Ok(func(user_index, vibration)) },
        Err(err) => Err(err.clone()),
    }
}
