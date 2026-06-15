#! /bin/sh

CRATEDIR=`dirname $0`/..

rm ${CRATEDIR}/src/ffi.rs
bindgen --formatter=none \
	--allowlist-function 'nvlist_.*' \
	--allowlist-function 'FreeBSD_nvlist_.*' \
	--blocklist-function 'FreeBSD_nvlist_add_stringv' \
	--allowlist-type nvlist_t \
	--allowlist-type FreeBSD_nvlist_t \
	--blocklist-type size_t \
	--blocklist-type __size_t \
	--blocklist-type va_list \
	--blocklist-type __uint64_t \
	--blocklist-type __builtin_va_list \
	--blocklist-type __va_list_tag \
	--opaque-type FILE \
	/usr/include/sys/nv.h |
sed -E	-e 's/pub fn FreeBSD_([a-zA-Z0-9_]+)/#[link_name = \"FreeBSD_\1\"]pub fn \1/g' \
	-e 's/pub type FreeBSD_([a-zA-Z0-9_]+)/pub type FreeBSD_\1 = \1;pub type \1/g' \
	>> ${CRATEDIR}/src/ffi.rs
	cargo fmt -- src/ffi.rs
