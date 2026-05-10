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
