#[cfg(feature = "ledger")]
extern "C" {
    fn check_app_canary();
    fn zemu_log_stack(ctx: *const u8);
    fn zemu_log(buf: *const u8);
    fn zemu_log_num(buf: *const u8, num: u32);
}

pub fn app_canary() {
    #[cfg(feature = "ledger")]
    unsafe {
        check_app_canary()
    }
}

pub fn zlog(_buf: &str) {
    #[cfg(feature = "ledger")]
    unsafe {
        zemu_log(_buf.as_bytes().as_ptr())
    }
}
pub fn zlog_stack(_buf: &str) {
    #[cfg(feature = "ledger")]
    unsafe {
        zemu_log_stack(_buf.as_bytes().as_ptr())
    }
}

pub fn zlog_num(buf: &str, num: u32) {
    #[cfg(feature = "ledger")]
    unsafe {
        zemu_log_num(buf.as_bytes().as_ptr(), num)
    }
}
