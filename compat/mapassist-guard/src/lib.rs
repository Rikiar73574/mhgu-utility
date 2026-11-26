//! Standalone wrapper; Utility embeds the same guard runtime.
#![no_std]
mod runtime;
use core::ptr;
#[unsafe(no_mangle)]
pub extern "C" fn forge_onLoad(p: *mut forge::PluginInitParams) {
    unsafe {
        (*p).required_version = forge::REQUIRED_VERSION;
        let n = b"MapAssist Role Guard\0";
        ptr::copy_nonoverlapping(n.as_ptr(), (p as *mut u8).add(6), n.len());
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn forge_onInit() {
    let _ = forge::log::init();
}
#[unsafe(no_mangle)]
pub extern "C" fn forge_onUpdate(dt:f32){runtime::mapassist_guard_tick(dt);}
