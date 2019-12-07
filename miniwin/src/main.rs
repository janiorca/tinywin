#![no_main]
#![no_std]

#![windows_subsystem = "windows"]
#[cfg(windows)] extern crate winapi;

use core::mem::MaybeUninit;
use core::panic::PanicInfo;

use winapi::shared::minwindef::{
    LRESULT,
    LPARAM,
    LPVOID,
    WPARAM,
    UINT,
};

use winapi::shared::windef::{
    HWND,
    HMENU,
    HICON,
    HBRUSH,
};

use winapi::um::libloaderapi::GetModuleHandleA;

use winapi::um::winuser::{
    DrawTextA,
    BeginPaint,
    EndPaint,
    GetClientRect,
    DefWindowProcA,
    RegisterClassA,
    CreateWindowExA,
    TranslateMessage,
    DispatchMessageA,
    GetMessageA,
    PostQuitMessage
};

use winapi::um::winuser::{
    WNDCLASSA,
    CS_OWNDC,
    CS_HREDRAW,
    CS_VREDRAW,
    CW_USEDEFAULT,
    WS_OVERLAPPEDWINDOW,
    WS_VISIBLE,
    DT_SINGLELINE,
    DT_CENTER,
    DT_VCENTER
};

pub unsafe extern "system" fn window_proc(hwnd: HWND,
    msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {

    match msg {
        winapi::um::winuser::WM_PAINT => {
            let mut paint_struct = MaybeUninit::uninit();
            let mut rect = MaybeUninit::uninit();
            let hdc = BeginPaint(hwnd, paint_struct.as_mut_ptr());
            GetClientRect(hwnd, rect.as_mut_ptr());
            DrawTextA(hdc, "Hello world\0".as_ptr() as *const i8, -1, rect.as_mut_ptr(), DT_SINGLELINE | DT_CENTER | DT_VCENTER);
            EndPaint(hwnd, paint_struct.as_mut_ptr());
        }
        winapi::um::winuser::WM_DESTROY => {
            PostQuitMessage(0);
        }
        _ => { return DefWindowProcA(hwnd, msg, wparam, lparam); }
    }
    return 0;
}

fn create_window( ) -> HWND {
    unsafe {
        let hinstance = GetModuleHandleA( 0 as *const i8 );
        let wnd_class = WNDCLASSA {
            style : CS_OWNDC | CS_HREDRAW | CS_VREDRAW,     
            lpfnWndProc : Some( window_proc ),
            hInstance : hinstance,
            lpszClassName : "MyClass\0".as_ptr() as *const i8,
            cbClsExtra : 0,									
            cbWndExtra : 0,
            hIcon: 0 as HICON,
            hCursor: 0 as HICON,
            hbrBackground: 0 as HBRUSH,
            lpszMenuName: 0 as *const i8,
        };
        RegisterClassA( &wnd_class );

        CreateWindowExA(
            0,									// dwExStyle 
            "MyClass\0".as_ptr() as *const i8,		                // class we registered.
            "MiniWIN\0".as_ptr() as *const i8,						// title
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,	// dwStyle
            CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT,	// size and position
            0 as HWND,               	// hWndParent
            0 as HMENU,					// hMenu
            hinstance,                  // hInstance
            0 as LPVOID )				// lpParam
    }
}

// More info: https://msdn.microsoft.com/en-us/library/windows/desktop/ms644927(v=vs.85).aspx
fn handle_message( window : HWND ) -> bool {
    unsafe {
        let mut msg = MaybeUninit::uninit();
        if GetMessageA( msg.as_mut_ptr(), window, 0, 0 ) > 0 {
                TranslateMessage( msg.as_ptr() );
                DispatchMessageA( msg.as_ptr() );
            true
        } else {
            false
        }
    }
}

#[panic_handler]
#[no_mangle]
pub extern fn panic( _info: &PanicInfo ) -> ! { loop {} }

#[no_mangle]
pub extern "system" fn mainCRTStartup() {
    let window = create_window(  );
    loop {
        if !handle_message( window ) {
            break;
        }
    }
}