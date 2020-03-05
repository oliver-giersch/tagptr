use std::env;

fn main() {
    if cfg!(target_arch = "x86_64") && !cfg!(feature = "nightly") {
        println!("building with external C library 'cmpxchg16b' support (stable/x86_64).");
        if env::var_os("CARGO_FEATURES_NIGHTLY").is_none() {
            cc::Build::new().file("extern/dwcas.c").compile("dwcas");
        }
    } else if cfg!(target_arch = "x86_64") {
        println!("building with stdsimd `cmpxchg16b` support (nightly/x86_64");
    } else {
        println!("building without without 'cmpxchg16' support (no x86_64).");
    }
}
