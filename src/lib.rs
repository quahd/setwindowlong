use std::ffi::c_void;
use std::ptr::null_mut;
use windows_sys::core::s;
use windows_sys::Win32::Foundation::{GetLastError, SetLastError, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::Diagnostics::Debug::OutputDebugStringA;
use windows_sys::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use windows_sys::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass};

use windows_sys::Win32::System::Threading::GetCurrentProcessId;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowThreadProcessId, IsWindowVisible, MessageBoxA, MessageBoxW,
    SetWindowLongPtrW, GWLP_WNDPROC, MB_OK, WM_KEYDOWN, WM_LBUTTONDOWN, WM_NCDESTROY,
    WM_RBUTTONDOWN,
};

pub fn vk_to_string(vk: usize) -> Option<String> {
    match vk {
        0x30..=0x39 | 0x41..=0x5A => char::from_u32(vk as u32).map(|c| c.to_string()),
        0x0D => Some("".to_string()),
        0x20 => Some(" ".to_string()),
        _ => None,
    }
}

unsafe extern "system" fn wnd_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_KEYDOWN => {
            let vk = vk_to_string(wparam);
            if vk.is_some() {
                let char = vk.unwrap();
                let content = format!(
                    "Detected message: {}\nMessage ID: 0x{:X} and char {:#?}",
                    WM_KEYDOWN, message, char
                );

                let text: Vec<u16> = content.encode_utf16().chain(std::iter::once(0)).collect();

                let title: Vec<u16> = "Message Detected"
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();

                MessageBoxW(window, text.as_ptr(), title.as_ptr(), MB_OK);
                Some("WM_KEYDOWN")
            } else {
                None
            }
        }
        _ => None,
    };

    DefSubclassProc(window, message, wparam, lparam)
}

// extern "system" fn wnd_proc(
//     window: HWND,
//     message: u32,
//     wparam: WPARAM,
//     lparam: LPARAM,
//     _: usize,
//     _: usize,
// ) -> LRESULT {
//     unsafe {
//         match message {
//             WM_PAINT => {
//                 let msg = b"ZOMG!";
//                 let mut ps = PAINTSTRUCT::default();
//                 let psp = &mut ps as *mut PAINTSTRUCT;
//                 let rectp = &mut ps.rcPaint as *mut RECT;
//                 let hdc = BeginPaint(window, psp);
//                 let brush = CreateSolidBrush(0x0000F0F0);
//                 FillRect(hdc, &ps.rcPaint, brush);
//                 DrawTextA(
//                     hdc,
//                     msg.as_ptr(),
//                     msg.len() as i32,
//                     rectp,
//                     DT_SINGLELINE | DT_CENTER | DT_VCENTER,
//                 );
//                 DeleteObject(brush);
//                 EndPaint(window, &ps);
//                 return 0;
//             }
//             WM_WINDOWPOSCHANGING => {
//                 let data = lparam as *mut WINDOWPOS;
//                 let data = data.as_mut().unwrap();
//                 data.flags |= SWP_NOSIZE | SWP_NOMOVE;
//                 return 0;
//             }
//             WM_NCDESTROY => {
//                 // let result = transmute::<WNDPROC, _>(PREV_WNDPROC);
//                 // SetWindowLongPtrW(window, GWLP_WNDPROC, result);
//                 RemoveWindowSubclass(window, Some(wnd_proc), 0);
//                 return DefWindowProcA(window, message, wparam, lparam);
//             }
//             _ => (),
//         }
//         // CallWindowProcW(PREV_WNDPROC, window, message, wparam, lparam)
//         DefSubclassProc(window, message, wparam, lparam)
//     }
// }

pub fn get_current_process_hwnd() -> Option<HWND> {
    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> i32 {
        let current_pid = GetCurrentProcessId();

        let mut window_pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut window_pid);

        if window_pid == current_pid && IsWindowVisible(hwnd) != 0 {
            let result = &mut *(lparam as *mut HWND);
            *result = hwnd;

            return 0; // Dừng EnumWindows
        }

        1 // Tiếp tục duyệt
    }

    unsafe {
        let mut hwnd: HWND = null_mut();

        EnumWindows(Some(enum_callback), &mut hwnd as *mut HWND as LPARAM);

        if hwnd.is_null() {
            None
        } else {
            Some(hwnd)
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    _module: *mut c_void,
    reason: u32,
    _reserved: *mut c_void,
) -> i32 {
    if reason == DLL_PROCESS_ATTACH {
        OutputDebugStringA(b"[AV-TEST] test_dll.dll loaded into Process B\0".as_ptr());
    }
    match reason {
        DLL_PROCESS_ATTACH => {
            std::thread::spawn(|| {
                attach();
            });
        }
        DLL_PROCESS_DETACH => (),
        _ => (),
    }

    1
}

fn attach() {
    unsafe {
        let hwnd_opt = get_current_process_hwnd();
        if hwnd_opt.is_none() {
            MessageBoxA(
                std::ptr::null_mut(),
                s!("Null handle!"),
                s!("hello.dll"),
                MB_OK,
            );
        } else {
            let hwnd = hwnd_opt.unwrap();
            SetLastError(0);
            let old = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, wnd_proc as isize);
            let error = GetLastError();
            if old == 0 && error != 0 {
                let code: Vec<u16> = format!(
                    "SetWindowLongPtrW failed: {}",
                    std::io::Error::from_raw_os_error(error as i32)
                )
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
                let title: Vec<u16> = "Window procedure error"
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();
                MessageBoxW(hwnd, code.as_ptr(), title.as_ptr(), MB_OK);
            }
        }
    };
}
