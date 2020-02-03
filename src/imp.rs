#[cfg(any(target_arch = "x86_64", target_arch = "powerpc64", target_arch = "aarch64"))]
mod arch64;

mod atomic;
mod non_null;
mod ptr;
