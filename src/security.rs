#![allow(unsafe_code)]

/// Durcit le processus courant contre les fuites mémoire.
/// Doit être appelé au tout début de main(), avant toute init Iced.
pub fn harden_process() {
  #[cfg(target_os = "linux")]
  {
    unsafe {
      // Désactive les core dumps — empêche la fuite de secrets sur SIGSEGV/SIGABRT.
      libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0);
      // Bloque ptrace par les processus du même uid (Yama LSM doit être actif).
      libc::prctl(libc::PR_SET_PTRACER, 0, 0, 0, 0);
    }
  }
}

/// Verrouille `len` octets en RAM à partir de `ptr` (empêche le swap).
/// No-op sur les plateformes non-unix.
#[cfg(unix)]
pub fn mlock_bytes(ptr: *const u8, len: usize) {
  unsafe {
    libc::mlock(ptr as *const libc::c_void, len);
  }
}

/// Déverrouille `len` octets à partir de `ptr`.
#[cfg(unix)]
pub fn munlock_bytes(ptr: *const u8, len: usize) {
  unsafe {
    libc::munlock(ptr as *const libc::c_void, len);
  }
}

#[cfg(not(unix))]
pub fn mlock_bytes(_ptr: *const u8, _len: usize) {}

#[cfg(not(unix))]
pub fn munlock_bytes(_ptr: *const u8, _len: usize) {}
