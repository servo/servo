# Storage

This crate contains Servo’s storage related implementations.

## IndexedDB

Servo currently includes two IndexedDB implementations that exist side by
side.

### Default implementation
It is enabled by default and is used in all normal Servo builds.

### Next generation implementation
An experimental redesign of IndexedDB. It lives alongside the current
implementation and can be enabled with a Cargo feature.

To build Servo with the new implementation:
`./mach build --features "indexeddb_next`

To run WPT tests with the new implementation:
```
./mach test-wpt tests/wpt/tests/IndexedDB
--metadata tests/wpt/indexeddb_next/meta
--log-raw /tmp/servo.log
```

To update expectations for the new implementation:
`./mach update-wpt --metadata tests/wpt/indexeddb_next/meta /tmp/servo.log`
