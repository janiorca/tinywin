#![no_main]
#![no_std]
#![windows_subsystem = "windows"]
#[cfg(windows)] extern crate winapi;

mod gl;
mod gl_util;

use gl::CVoid;
use core::mem::MaybeUninit;
use core::mem::size_of;
use core::panic::PanicInfo;
    
use winapi::um::wingdi::{
    ChoosePixelFormat,
    SwapBuffers,
    wglMakeCurrent,
    wglCreateContext,
    SetPixelFormat,

    PFD_TYPE_RGBA,
    PFD_DOUBLEBUFFER,
    PFD_SUPPORT_OPENGL,
    PFD_DRAW_TO_WINDOW,
    PIXELFORMATDESCRIPTOR
};

use winapi::shared::minwindef::{
    LRESULT,
    LPARAM,
    LPVOID,
    WPARAM,
    UINT,
};

use winapi::shared::windef::{
    HDC,
    HGLRC,
    HWND,
    HMENU,
    HICON,
    HBRUSH,
};

use winapi::um::libloaderapi::GetModuleHandleA;

use winapi::um::winuser::{
    CreateWindowExA,
    DefWindowProcA,
    DispatchMessageA,
    GetDC,
    PostQuitMessage,
    RegisterClassA,
    TranslateMessage,
    PeekMessageA,
    MessageBoxA,

    MB_ICONERROR,
    MSG,
    WNDCLASSA,
    CS_OWNDC,
    CS_HREDRAW,
    CS_VREDRAW,
    CW_USEDEFAULT,
    PM_REMOVE, 
    WS_OVERLAPPEDWINDOW,
    WS_VISIBLE,
};

pub unsafe extern "system" fn window_proc(hwnd: HWND,
    msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {

    match msg {
        winapi::um::winuser::WM_DESTROY => {
            PostQuitMessage(0);
        }
        _ => { return DefWindowProcA(hwnd, msg, w_param, l_param); }
    }
    return 0;
}

fn show_error( message : *const i8 ) {
    unsafe{
        MessageBoxA(0 as HWND, message, "Window::create\0".as_ptr() as *const i8, MB_ICONERROR);
    }
}

// Create window function 
// https://mariuszbartosik.com/opengl-4-x-initialization-in-windows-without-a-framework/
fn create_window( ) -> ( HWND, HDC ) {
    unsafe {
        let hinstance = GetModuleHandleA( 0 as *const i8 );
        let wnd_class = WNDCLASSA {
            style : CS_OWNDC | CS_HREDRAW | CS_VREDRAW,     
            lpfnWndProc : Some( window_proc ),
            hInstance : hinstance,							// The instance handle for our application which we can retrieve by calling GetModuleHandleW.
            lpszClassName : "MyClass\0".as_ptr() as *const i8,
            cbClsExtra : 0,									
            cbWndExtra : 0,
            hIcon: 0 as HICON,
            hCursor: 0 as HICON,
            hbrBackground: 0 as HBRUSH,
            lpszMenuName: 0 as *const i8,
        };
        RegisterClassA( &wnd_class );

        // More info: https://msdn.microsoft.com/en-us/library/windows/desktop/ms632680(v=vs.85).aspx
        let h_wnd = CreateWindowExA(
            0,
            //WS_EX_APPWINDOW | WS_EX_WINDOWEDGE,                     // dwExStyle 
            "MyClass\0".as_ptr() as *const i8,		                // class we registered.
            "GLWIN\0".as_ptr() as *const i8,						// title
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,	// dwStyle
            CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT,	// size and position
            0 as HWND,               	// hWndParent
            0 as HMENU,					// hMenu
            hinstance,                  // hInstance
            0 as LPVOID );				// lpParam
        
        let h_dc : HDC = GetDC(h_wnd);        // Device Context            

        let mut pfd : PIXELFORMATDESCRIPTOR = core::mem::zeroed();
        pfd.nSize = core::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16;
        pfd.nVersion = 1;
        pfd.dwFlags = PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER;
        pfd.iPixelType = PFD_TYPE_RGBA;
        pfd.cColorBits = 32;
        pfd.cAlphaBits = 8;
        pfd.cDepthBits = 32;
         
        let pf_id : i32 = ChoosePixelFormat(h_dc, &pfd );
        if pf_id == 0 {
            show_error( "ChoosePixelFormat() failed.\0".as_ptr() as *const i8);
            return ( 0 as HWND, h_dc ) ;
        }

        if SetPixelFormat(h_dc, pf_id, &pfd) == 0  {
            show_error( "SetPixelFormat() failed.\0".as_ptr() as *const i8);
            return ( 0 as HWND, h_dc ) ;
        }

        let gl_context : HGLRC = wglCreateContext(h_dc);    // Rendering Contex
        if gl_context == 0 as HGLRC {
            show_error( "wglCreateContext() failed.\0".as_ptr() as *const i8 );
            return ( 0 as HWND, h_dc ) ;
        }
         
        if wglMakeCurrent(h_dc, gl_context) == 0 {
            show_error( "wglMakeCurrent() failed.\0".as_ptr() as *const i8);
            return ( 0 as HWND, h_dc ) ;
        }
        gl::init();
        gl::wglSwapIntervalEXT(1);
        ( h_wnd, h_dc )
    }
}

// Create message handling function with which to link to hook window to Windows messaging system
// More info: https://msdn.microsoft.com/en-us/library/windows/desktop/ms644927(v=vs.85).aspx
fn handle_message( _window : HWND ) -> bool {
    unsafe {
       let mut msg : MSG = MaybeUninit::uninit().assume_init();
        loop{
            if PeekMessageA( &mut msg,0 as HWND,0,0,PM_REMOVE) == 0 {
                return true;
            }
            if msg.message == winapi::um::winuser::WM_QUIT {
                return false;
            }
            TranslateMessage( &msg  );
            DispatchMessageA( &msg  );
        }
    }
}


#[panic_handler]
#[no_mangle]
pub extern fn panic( _info: &PanicInfo ) -> ! { loop {} }

#[no_mangle]
pub unsafe extern fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *((dest as usize + i) as *mut u8) = c as u8;
        i += 1;
    }
    dest
}

#[no_mangle]
pub unsafe extern fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *((dest as usize + i) as *mut u8) = *((src as usize + i) as *const u8);
        i += 1;
    }
    dest
}

#[no_mangle]
pub extern "system" fn mainCRTStartup() {
    let ( window, hdc ) = create_window(  );
    let mut error_message : [i8;1000] = [ 0; 1000];

    let vtx_shader_src : &'static str = "#version 330 core
    layout (location = 0) in vec3 Position;
    void main()
    {
     gl_Position = vec4(Position, 1.0);
    }\0";

    let frag_shader_src : &'static str = "#version 330 core
    #define num_circles 4
    in vec4 gl_FragCoord;
    out vec4 Color;
    uniform float iTime;
    void main()
    {
        vec2 uv = gl_FragCoord.xy/800.0;
        vec2 sc = vec2( 0.5, 0.5 ); 
        vec4 fragColor = vec4( 0.0, 0.0, 0.0, 0.0 );
        Color = vec4( 0.0, 0.0, 0.0, 0.0 );
        for( int idx=0; idx<num_circles; idx++ ) 
        {
            vec2 center = vec2(sin( iTime*(float(idx)*0.132+0.1672 ) )*(0.146+0.0132*float(idx)),
                                    sin( iTime+ iTime*(float(idx)*0.1822+0.221))*(0.1131+0.0112*float(idx)) ) + sc;

            float dist = distance( center, uv );
            vec3 col = vec3( sin( 100.0*dist ), sin( 110.0*dist ), sin( 120.0*dist ) );
            col *= max( 0.0, (1.0-dist*3.0) );
            Color += vec4( col, 0.0 );
        }    
    }\0\0";

    let vtx_coords : [ [ gl::GLfloat; 3 ]; 4 ] = [
        [ -1.0, -1.0, 0.0 ],
        [ 1.0, -1.0, 0.0 ],
        [ -1.0,  1.0, 0.0 ],
        [ 1.0,  1.0, 0.0 ],
     ];
   
    let vtx_shader = match gl_util::shader_from_source( vtx_shader_src, gl::VERTEX_SHADER, &mut error_message ) {
        Some( shader ) => shader,
        None => { show_error( error_message.as_ptr()  ); 0 }
    };

    let frag_shader  = match gl_util::shader_from_source( frag_shader_src, gl::FRAGMENT_SHADER,  &mut error_message ) {
        Some( shader ) => shader,
        None => { show_error( error_message.as_ptr() ); 0 }
    };

    let shader_prog = match gl_util::program_from_shaders(vtx_shader, frag_shader, &mut error_message ) {
        Some( prog ) => prog,
        None => { show_error( error_message.as_ptr() ); 0 }
    };

    let mut vertex_buffer_id : gl::GLuint = 0;
    let mut vertex_array_id : gl::GLuint = 0;
    unsafe{
        // Generate 1 buffer, put the resulting identifier in vertexbuffer
        gl::GenBuffers(1, &mut vertex_buffer_id);
  
        gl::GenVertexArrays(1, &mut vertex_array_id );
        gl::BindVertexArray(vertex_array_id);
  
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_id);
        gl::BufferData( gl::ARRAY_BUFFER, size_of::<gl::GLfloat>() as isize * 3 * 4, vtx_coords.as_ptr() as *const gl::CVoid, gl::STATIC_DRAW);

        gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
        gl::VertexAttribPointer(
            0, // index of the generic vertex attribute ("layout (location = 0)")
            3, // the number of components per generic vertex attribute

            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            (3 * core::mem::size_of::<f32>()) as gl::GLint, // stride (byte offset between consecutive attributes)
            0 as *const CVoid // offset of the first component
        );    
    }

    let mut time : f32 = 0.0;
    loop {
        if !handle_message( window ) {
            break;
        }
        unsafe{
            let rgba = &[ 0.4f32, 1.0, 0.9, 0.0 ];
            gl::ClearBufferfv(gl::COLOR, 0, rgba as *const _ );  

            gl::UseProgram(shader_prog);
  
           let vertex_color_loc : i32 = gl::GetUniformLocation(shader_prog, "iTime\0".as_ptr());
           gl::Uniform1f(vertex_color_loc, time );
  
            gl::BindVertexArray(vertex_array_id);
            gl::DrawArrays( gl::TRIANGLE_STRIP, 0, 4 );
            SwapBuffers(hdc);
            time += 1.0 / 60.0f32;            
        }
    }
    unsafe{
        // Tying to exit normally seems to crash after certain APIs functions have been called. ( Like ChoosePixelFormat )
        winapi::um::processthreadsapi::ExitProcess(0);
    }
}

// Compiling with no_std seems to require the following symbol to be set if there is any floating point code anywhere in the code
#[no_mangle]
pub static _fltused : i32 = 1;
