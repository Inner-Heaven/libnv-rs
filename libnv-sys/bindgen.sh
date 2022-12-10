#! /bin/sh
bindgen --generate functions,types \
	--allowlist-type 'nvlist_t' \
	--allowlist-function 'nvlist_.*' \
	/usr/include/sys/nv.h > src/lib.rs
