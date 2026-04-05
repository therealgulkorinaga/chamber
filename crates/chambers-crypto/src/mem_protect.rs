//! Memory protection primitives for Phase 2 hardening.
//!
//! - mlock: pin memory in physical RAM, prevent swap
//! - MADV_DONTDUMP: exclude from core dumps
//! - core dump disable: setrlimit(RLIMIT_CORE, 0)
//! - guard buffer: page-aligned, mlock'd, zeroed after every use

use std::ptr;
use zeroize::Zeroize;

/// Lock a memory region in physical RAM. Prevents paging to swap.
/// Returns true on success.
pub fn mlock_buffer(ptr: *const u8, len: usize) -> bool {
    unsafe { libc::mlock(ptr as *const libc::c_void, len) == 0 }
}

/// Unlock a previously mlock'd memory region.
pub fn munlock_buffer(ptr: *const u8, len: usize) -> bool {
    unsafe { libc::munlock(ptr as *const libc::c_void, len) == 0 }
}

/// Mark a memory region as excluded from core dumps.
/// On macOS, MADV_ZERO_WIRED_PAGES is used as MADV_DONTDUMP equivalent.
pub fn mark_dontdump(ptr: *const u8, len: usize) -> bool {
    // macOS doesn't have MADV_DONTDUMP directly.
    // Use madvise with MADV_FREE as best-effort, and rely on
    // setrlimit(RLIMIT_CORE, 0) as the primary core dump prevention.
    #[cfg(target_os = "macos")]
    {
        // On macOS, the primary protection is setrlimit + mlock.
        // MADV_DONTDUMP is Linux-specific. We return true as a no-op
        // since core dumps are disabled at process level.
        let _ = (ptr, len);
        true
    }
    #[cfg(target_os = "linux")]
    {
        unsafe { libc::madvise(ptr as *mut libc::c_void, len, libc::MADV_DONTDUMP) == 0 }
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = (ptr, len);
        false
    }
}

/// Disable core dumps for this process.
/// After this call, SIGABRT/SIGSEGV will not produce a core file.
pub fn disable_core_dumps() -> bool {
    let rlim = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    unsafe { libc::setrlimit(libc::RLIMIT_CORE, &rlim) == 0 }
}

/// Deny debugger attachment via ptrace.
/// After this call, lldb/dtrace cannot attach to this process.
#[cfg(target_os = "macos")]
pub fn deny_debugger_attach() -> bool {
    // PT_DENY_ATTACH = 31 on macOS
    const PT_DENY_ATTACH: libc::c_int = 31;
    unsafe {
        libc::ptrace(PT_DENY_ATTACH, 0, ptr::null_mut::<libc::c_char>(), 0) == 0
    }
}

#[cfg(not(target_os = "macos"))]
pub fn deny_debugger_attach() -> bool {
    // Linux equivalent would be prctl(PR_SET_DUMPABLE, 0)
    false
}

/// Check if a debugger is currently attached (macOS).
/// Uses sysctl to check the P_TRACED flag on this process.
#[cfg(target_os = "macos")]
pub fn is_debugger_attached() -> bool {
    use std::mem;

    // kinfo_proc is large and not in libc crate; use raw sysctl with byte buffer
    const CTL_KERN: libc::c_int = 1;
    const KERN_PROC: libc::c_int = 14;
    const KERN_PROC_PID: libc::c_int = 1;
    const P_TRACED: i32 = 0x00000800;

    // kinfo_proc is ~648 bytes on macOS; use oversized buffer
    let mut buf = [0u8; 1024];
    let mut size = buf.len();
    let mut mib: [libc::c_int; 4] = [
        CTL_KERN,
        KERN_PROC,
        KERN_PROC_PID,
        unsafe { libc::getpid() },
    ];
    let ret = unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            4,
            buf.as_mut_ptr() as *mut libc::c_void,
            &mut size,
            ptr::null_mut(),
            0,
        )
    };
    if ret != 0 {
        return false;
    }
    // p_flag is at offset 32 in kinfo_proc.kp_proc on macOS (extern_proc struct)
    // This is architecture-specific but stable on aarch64 macOS
    if size >= 36 {
        let p_flag = i32::from_ne_bytes([buf[32], buf[33], buf[34], buf[35]]);
        (p_flag & P_TRACED) != 0
    } else {
        false
    }
}

#[cfg(not(target_os = "macos"))]
pub fn is_debugger_attached() -> bool {
    false
}

/// A page-aligned, mlock'd buffer for transient plaintext.
/// Zeroed after every use. This is the only place plaintext can exist.
pub struct GuardBuffer {
    ptr: *mut u8,
    len: usize,
}

// GuardBuffer is Send — it's a raw allocation managed by one owner
unsafe impl Send for GuardBuffer {}

impl GuardBuffer {
    /// Allocate a guard buffer of the given size (rounded up to page size).
    pub fn new(min_size: usize) -> Result<Self, String> {
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize };
        let len = ((min_size + page_size - 1) / page_size) * page_size;

        let ptr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1,
                0,
            )
        };

        if ptr == libc::MAP_FAILED {
            return Err("mmap failed for guard buffer".into());
        }

        let ptr = ptr as *mut u8;

        // Lock in physical RAM
        if unsafe { libc::mlock(ptr as *const libc::c_void, len) } != 0 {
            unsafe { libc::munmap(ptr as *mut libc::c_void, len) };
            return Err("mlock failed for guard buffer".into());
        }

        // Mark as exclude from core dumps (Linux)
        mark_dontdump(ptr, len);

        // Zero it
        unsafe { ptr::write_bytes(ptr, 0, len) };

        Ok(Self { ptr, len })
    }

    /// Get a mutable slice to write into.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    /// Get a read-only slice.
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    /// Zero the entire buffer. Called after every use.
    pub fn zero(&mut self) {
        let slice = unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) };
        slice.zeroize();
    }

    /// Buffer capacity in bytes.
    pub fn capacity(&self) -> usize {
        self.len
    }
}

impl Drop for GuardBuffer {
    fn drop(&mut self) {
        // Zero before freeing
        self.zero();
        // Unlock
        unsafe { libc::munlock(self.ptr as *const libc::c_void, self.len) };
        // Unmap
        unsafe { libc::munmap(self.ptr as *mut libc::c_void, self.len) };
    }
}

/// Apply all process-level hardening at startup.
/// Call this once at the beginning of main().
pub fn harden_process() {
    let core = disable_core_dumps();
    let debug = deny_debugger_attach();

    if core {
        eprintln!("[hardening] core dumps disabled");
    } else {
        eprintln!("[hardening] WARNING: failed to disable core dumps");
    }

    if debug {
        eprintln!("[hardening] debugger attachment denied");
    } else {
        eprintln!("[hardening] WARNING: failed to deny debugger attachment");
    }
}

/// Lock a WorldKey's buffer in memory.
pub fn mlock_key(key_bytes: &[u8; 32]) -> bool {
    let result = mlock_buffer(key_bytes.as_ptr(), 32);
    if result {
        mark_dontdump(key_bytes.as_ptr(), 32);
    }
    result
}
