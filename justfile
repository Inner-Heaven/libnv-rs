workspace := "~/libnv-rs"
ubuntu_host := "zetta-ubuntu"
rsync_exclude := "--exclude .git --exclude .idea --exclude target --exclude libzfs_core-sys/target"

set positional-arguments
test-ubuntu args='':
    just copy-code-to {{ubuntu_host}}
    ssh {{ubuntu_host}} '. "$HOME/.cargo/env";cd {{workspace}} && cargo test --no-default-features --features nvpair {{args}}'

copy-code-to host:
 rsync -az -e "ssh" {{rsync_exclude}} --progress ./ {{host}}:{{workspace}}


