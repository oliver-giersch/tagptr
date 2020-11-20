fn main() {
    if cfg!(target_arch = "x86_64") {
        if cfg!(feature = "nightly") {
            println!("building with stdsimd `cmpxchg16b` support (nightly/x86_64)");
        } else {
            println!("building with external C library 'cmpxchg16b' support (stable/x86_64).");
            cc::Build::new().file("extern/dwcas.c").compile("dwcas");
            println!("cargo:rustc-link-lib=atomic");
        }
    } else {
        println!("building without without 'cmpxchg16' support (no x86_64).");
    }
}
