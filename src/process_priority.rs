#[cfg(target_os = "windows")]
mod priority {
    use winapi::um::processthreadsapi::{GetCurrentProcess, SetPriorityClass};
    use winapi::um::winbase::REALTIME_PRIORITY_CLASS;

    pub fn set_high_priority() {
        unsafe {
            let process = GetCurrentProcess();
            if SetPriorityClass(process, REALTIME_PRIORITY_CLASS) == 0 {
                eprintln!("Warning: Failed to set process priority.");
            }
        }
    }
}

#[cfg(target_family = "unix")]
mod priority {
    use libc::{PRIO_PROCESS, setpriority};
    use std::process;

    pub fn set_high_priority() {
        unsafe {
            let pid = process::id();
            if setpriority(PRIO_PROCESS, pid, -20) != 0 {
                eprintln!("Warning: Failed to set high process priority.");
            }
        }
    }
}

#[cfg(not(any(target_os = "windows", target_family = "unix")))]
mod priority {
    pub fn set_high_priority() {
        eprintln!("Warning: Setting process priority is not supported on this platform.");
    }
}

// Public function to call from main code
pub fn set_high_priority() {
    priority::set_high_priority();
}
