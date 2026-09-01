use std::os::raw::{c_long, c_uint, c_ulong, c_ushort, c_void};

#[repr(C)]
struct COORD {
    x: c_ushort,
    y: c_ushort,
}

#[repr(C)]
struct CONSOLE_FONT_INFOEX {
    cb_size: c_ulong,
    n_font: c_ulong,
    dw_font_size: COORD,
    font_family: c_uint,
    font_weight: c_uint,
    face_name: [u16; 32],
}

extern "system" {
    fn GetStdHandle(nStdHandle: c_ulong) -> *mut c_void;
    fn SetCurrentConsoleFontEx(
        hConsoleOutput: *mut c_void,
        bMaximumWindow: c_long,
        lpConsoleCurrentFontEx: *mut CONSOLE_FONT_INFOEX,
    ) -> c_long;
    fn GetConsoleMode(hConsoleHandle: *mut c_void, lpMode: *mut c_ulong) -> c_long;
    fn SetConsoleMode(hConsoleHandle: *mut c_void, dwMode: c_ulong) -> c_long;
    fn SetConsoleTitleW(lpConsoleTitle: *const u16) -> c_long;
}

const STD_OUTPUT_HANDLE: c_ulong = 0xFFFFFFF5;
const ENABLE_VIRTUAL_TERMINAL_PROCESSING: c_ulong = 0x0004;

pub fn set(window_title: &str) {
    unsafe {
        std::process::Command::new("cmd")
            .args(["/C", "color 0A"])
            .status()
            .ok();
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        // Enable VT processing so ANSI escapes (incl. OSC 8 hyperlinks) work in conhost.
        let mut mode: c_ulong = 0;
        if GetConsoleMode(handle, &mut mode) != 0 {
            SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
        }
        let mut font = CONSOLE_FONT_INFOEX {
            cb_size: std::mem::size_of::<CONSOLE_FONT_INFOEX>() as c_ulong,
            n_font: 0,
            dw_font_size: COORD { x: 0, y: 20 },
            font_family: 54,
            font_weight: 700,
            face_name: [0; 32],
        };
        let face = "Consolas";
        for (i, c) in face.encode_utf16().enumerate() {
            font.face_name[i] = c;
        }
        SetCurrentConsoleFontEx(handle, 0, &mut font);
        // Set the window title bar so version is visible without taking menu space.
        let mut title_utf16: Vec<u16> = window_title.encode_utf16().collect();
        title_utf16.push(0);
        SetConsoleTitleW(title_utf16.as_ptr());
    }
}
