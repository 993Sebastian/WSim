//! The boundary to JavaScript. Requests and answers are byte buffers in the module's
//! memory: JavaScript reserves a buffer with `wsim_alloc`, writes into it and hands it
//! over; answers come back as buffer address and length packed into one number and are
//! released with `wsim_free`. Every buffer is a boxed byte slice, so that its length is
//! all it takes to release it.
#![allow(unsafe_code)]

use std::ptr::slice_from_raw_parts_mut;

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    /// Progress of a round, for the progress bar.
    fn wsim_fortschritt(done: u32, total: u32);
    /// Text of a panic, before the module traps.
    fn wsim_absturz(ptr: *const u8, len: usize);
}

/// Hands a buffer to JavaScript: address in the upper, length in the lower 32 bits.
fn abgeben(bytes: Vec<u8>) -> u64 {
    let buffer = bytes.into_boxed_slice();
    let len = buffer.len() as u64;
    let ptr = Box::into_raw(buffer).cast::<u8>() as usize as u64;
    (ptr << 32) | len
}

/// Reads a buffer JavaScript filled and gave back; it is released afterwards.
///
/// # Safety
/// `ptr` and `len` must come from `wsim_alloc(len)`.
unsafe fn uebernehmen(ptr: *mut u8, len: usize) -> Vec<u8> {
    // SAFETY: the caller hands over a buffer from `wsim_alloc` with its length.
    unsafe { Box::from_raw(slice_from_raw_parts_mut(ptr, len)) }.into_vec()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Reserves a buffer of `len` bytes for JavaScript to write into.
#[unsafe(no_mangle)]
pub extern "C" fn wsim_alloc(len: usize) -> *mut u8 {
    Box::into_raw(vec![0u8; len].into_boxed_slice()).cast::<u8>()
}

/// Releases a buffer from `wsim_alloc` or an answer.
///
/// # Safety
/// `ptr` and `len` must describe a buffer this module handed out and not yet released.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wsim_free(ptr: *mut u8, len: usize) {
    // SAFETY: see above; the buffer was a boxed slice of this length.
    drop(unsafe { Box::from_raw(slice_from_raw_parts_mut(ptr, len)) });
}

/// Installs the panic report; called once after loading the module.
#[unsafe(no_mangle)]
pub extern "C" fn wsim_start() {
    std::panic::set_hook(Box::new(|info| {
        let message = info.to_string();
        // SAFETY: JavaScript only reads the text before the module traps.
        unsafe { wsim_absturz(message.as_ptr(), message.len()) };
    }));
}

/// Answers a JSON request (see `crate::anfrage`); takes over the request buffer.
///
/// # Safety
/// `ptr` and `len` must come from `wsim_alloc(len)`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wsim_anfrage(ptr: *mut u8, len: usize) -> u64 {
    // SAFETY: forwarded from the caller.
    let request = text(&unsafe { uebernehmen(ptr, len) });
    let answer = crate::anfrage(&request, &mut |done, total| {
        // SAFETY: a plain call into JavaScript with two numbers.
        unsafe { wsim_fortschritt(done, total) }
    });
    abgeben(answer.into_bytes())
}

/// The bytes of a save (empty if it does not exist); takes over the name buffer.
///
/// # Safety
/// `ptr` and `len` must come from `wsim_alloc(len)`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wsim_spielstand(ptr: *mut u8, len: usize) -> u64 {
    // SAFETY: forwarded from the caller.
    let name = text(&unsafe { uebernehmen(ptr, len) });
    abgeben(crate::spielstand(&name).unwrap_or_default())
}

/// Adds a save from the browser's storage; takes over both buffers.
///
/// # Safety
/// Both pairs must come from `wsim_alloc(len)`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wsim_spielstand_einlegen(
    name_ptr: *mut u8,
    name_len: usize,
    data_ptr: *mut u8,
    data_len: usize,
) {
    // SAFETY: forwarded from the caller.
    let name = text(&unsafe { uebernehmen(name_ptr, name_len) });
    // SAFETY: forwarded from the caller.
    let bytes = unsafe { uebernehmen(data_ptr, data_len) };
    crate::spielstand_einlegen(&name, bytes);
}
