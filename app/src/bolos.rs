#[cfg(feature = "ledger-debug")]
extern "C" {
    fn check_app_canary();
    fn zemu_log_stack(ctx: *const u8);
    fn zemu_log(buf: *const u8);
    fn zemu_log_num(buf: *const u8, num: u32);
}

pub fn app_canary() {
    #[cfg(feature = "ledger-debug")]
    unsafe {
        check_app_canary()
    }
}

pub fn zlog(_buf: &str) {
    #[cfg(feature = "ledger-debug")]
    unsafe {
        zemu_log(_buf.as_bytes().as_ptr())
    }
    #[cfg(test)]
    std::println!("{_buf}")
}

pub fn zlog_stack(_buf: &str) {
    #[cfg(feature = "ledger-debug")]
    unsafe {
        zemu_log_stack(_buf.as_bytes().as_ptr())
    }
    #[cfg(test)]
    std::println!("{_buf}")
}

pub fn zlog_num(_buf: &str, _num: u32) {
    #[cfg(feature = "ledger-debug")]
    unsafe {
        zemu_log_num(_buf.as_bytes().as_ptr(), _num)
    }
}
