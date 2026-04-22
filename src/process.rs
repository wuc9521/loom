use std::sync::Mutex;

static PIDS: Mutex<Vec<u32>> = Mutex::new(Vec::new());

pub fn register_pid(pid: u32) {
    PIDS.lock().unwrap().push(pid);
}

pub fn unregister_pid(pid: u32) {
    let mut pids = PIDS.lock().unwrap();
    if let Some(pos) = pids.iter().position(|&x| x == pid) {
        pids.remove(pos);
    }
}

pub fn cleanup_children() {
    let mut pids = PIDS.lock().unwrap();
    for pid in pids.drain(..) {
        #[cfg(unix)]
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
        #[cfg(windows)]
        {
            // Windows-specific cleanup could go here
        }
    }
}
