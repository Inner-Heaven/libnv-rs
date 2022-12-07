use std::{
    env,
    fs,
    path::Path,
    os::unix
};

fn main() {
    // FreeBSD 14 adds a libnv.so.1, which is the same as libnv.so.0 except that
    // its symbols are renamed so as not to collide with libnvpair.so's.  Always
    // link to libnv.so.0.  That way out binaries will be cross-compilable and
    // run on FreeBSD 11.0 or greater, and we can use the same symbol names for
    // all OS versions.
    let out_dir = env::var("OUT_DIR").unwrap();
    let link = Path::join(Path::new(&out_dir), "libnv.so");
    match fs::read_link(&link) {
        Ok(l) if l == link => (),
        Ok(_) => {
            fs::remove_file(&link).unwrap();
            unix::fs::symlink("/lib/libnv.so.0", &link).unwrap();
        },
        Err(_) => {
            unix::fs::symlink("/lib/libnv.so.0", &link).unwrap();
        }
    }
    println!("cargo:rerun-if-env-changed=OUT_DIR");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-search=native={out_dir}");

    // Link to libnv
    println!("cargo:rustc-link-lib=nv");
}
