# Bluetooth Rust lib using macOS CoreBluetooth

[![Build Status](https://travis-ci.org/akosthekiss/blurmac.svg?branch=master)](https://travis-ci.org/akosthekiss/blurmac)
[![Crates.io](https://img.shields.io/crates/v/blurmac.svg)](https://crates.io/crates/blurmac)

The main aim of BlurMac is to enable [WebBluetooth](https://webbluetoothcg.github.io)
in [Servo](https://github.com/servo/servo) on macOS. Thus, API and implementation
decisions are affected by the encapsulating [Devices](https://github.com/servo/devices),
and the sibling [BlurZ](https://github.com/szeged/blurz) and [BlurDroid](https://github.com/szeged/blurdroid)
crates.


## Run Servo with WebBluetooth Enabled

Usually, you don't want to work with BlurMac on its own but use it within Servo.
So, most probably you'll want to run Servo with WebBluetooth enabled:

```
RUST_LOG=blurmac \
./mach run \
    --dev \
    --pref=dom.bluetooth.enabled \
    --pref=dom.permissions.testing.allowed_in_nonsecure_contexts \
    URL
```

Notes:
* The above command is actually not really BlurMac-specific (except for the `RUST_LOG`
  part). It runs Servo with WBT enabled on any platform where WBT is supported.
* You don't need the `RUST_LOG=blurmac` part if you don't want to see BlurMac debug
  messages on the console.
* You don't need the `--dev` part if you want to run a release build.
* You don't need the `--pref=dom.permissions.testing.allowed_in_nonsecure_contexts`
  part if your `URL` is https (but you do need it if you test a local file).


## Known Issues

* Device RSSI can not be retrieved yet.
* Support for included services is incomplete.
* Descriptors are not supported yet.
* Notifications on characteristics are not supported yet (the limitation comes from
  Devices).


## Compatibility

Tested on:

* macOS Sierra 10.12.


## Copyright and Licensing

Licensed under the BSD 3-Clause [License](LICENSE.md).
