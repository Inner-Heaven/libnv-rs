#[cfg(target_os = "freebsd")]
fn main() {
    println!("cargo:rustc-link-lib=nv");
    println!("cargo:rustc-check-cfg=cfg(crossdocs)")
}

#[cfg(not(target_os = "freebsd"))]
fn main() {
    // If we're building not on FreeBSD, there's no way the build can succeed.
    // This probably means we're building docs on docs.rs, so set this config
    // variable.  We'll use it to stub out the crate well enough that
    // libnv's docs can build.
    println!("cargo:rustc-cfg=crossdocs");
    println!("cargo::rustc-check-cfg=cfg(crossdocs)");

    println!("cargo:rustc-check-cfg=cfg(crossdocs)")
}
