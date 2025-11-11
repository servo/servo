# Storage

This crate contains Servo’s storage related implementations.

## IndexedDB

There are currently two IndexedDB implementations:

- Default implementation
  Enabled by default and used for all normal Servo builds.

- Next-generation implementation
  An experimental redesign that lives alongside the current one.

  It can be enabled using a Cargo feature:

  `./mach build --features "indexeddb_next`
