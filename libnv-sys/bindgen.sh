#! /bin/sh
# Must be run on FreeBSD 13 or lower.  FreeBSD 14's libnv has a higher .so
# version, and uses different symbol names.  For backwards compatibility, we use
# libnv.so.0.  See also build.rs.

cat > src/lib.rs << HERE
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
HERE

bindgen --generate functions,types \
	--allowlist-type 'nvlist_t' \
	--allowlist-function 'nvlist_.*' \
	/usr/include/sys/nv.h >> src/lib.rs
