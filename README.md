# LockedBox

A crate providing an `mlock`-locked `Box<T>`, preventing the contents of its
memory from being paged to disk.

This crate prevents potential accidental unlocking of memory by ensuring the
memory allocated by `LockedBox<T>` is always a multiple of the operating
system's page size. Because `mlock`/`munlock` operate on pages of memory, this
guarantees that each `LockedBox<T>` is guaranteed to have its own lock status.

`LockedBox<T>` is a thin, safe abstraction built atop
[`memsec`](https://github.com/quininer/memsec).

## Alternatives

- [`region`](https://github.com/darfink/region-rs)

## Open-source Licenses

This project, like all projects from [Khonsu Labs](https://khonsulabs.com/), is open-source.
This repository is available under the [MIT License](./LICENSE-MIT) or the
[Apache License 2.0](./LICENSE-APACHE).

To learn more about contributing, please see [CONTRIBUTING.md](./CONTRIBUTING.md).
