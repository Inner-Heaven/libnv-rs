![libnv](libnv.png)

[![Crates.io](https://img.shields.io/crates/v/libnv.svg)](https://crates.io/crates/libwhisper)
> Rustic bindings to libnv.

## What's that?
This library is safe rust bindings to FreeBSD's Name/value pairs library ([`libnv`](man)). It's different from `libnvpair` from zfs project in a way that it has stable API.
You probably don't need it.
## Installation
If you have FreeBSD you already have library available in base system. If don't — well... figure out how to install it and send me a PR?

`libnv` is available on crates.io and can be included in your Cargo enabled project like this:

```
[dependencies]
libnv= "0.1"
```
## Usage
Read the [docs](https://docs.rs/libnv).



[man]: https://www.freebsd.org/cgi/man.cgi?query=nv
