use std::env;

fn main() {
    if cfg!(target_arch = "x86") && !cfg!(feature = "nightly") {
        println!("building with 'cmpxchg16b' support (x86_64).");
        if env::var_os("CARGO_FEATURES_NIGHTLY").is_none() {
            cc::Build::new().file("extern/dwcas.c").compile("dwcas");
        }
    } else {
        println!("building without 'cmpxchg16' support (no x86_64).");
    }
}
