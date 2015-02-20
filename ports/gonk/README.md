# Instructions for building the Gonk port

## Set up an android toolchain and NDK

Follow the steps [here](https://github.com/servo/servo/wiki/Building-for-Android) for setting up the Android NDK
and toolchain.
## Build B2G

Note: this will take a long time and will take around 20GB of space

Disable the screen timeout on the device, and connect to wifi. Also connect it to your computer
with remote debugging enabled.

```
git clone https://github.com/mozilla-b2g/B2G
./config.sh flame-kk
```

If behind a firewall, put the following in your gitconfig:

```
[url "https://github"]
        insteadOf = git://github
[url "https://git.mozilla.org/external/caf"]
    insteadOf = git://codeaurora.org
```
Then run

```
./build.sh libssl libsuspend libz libGLESv2 toolbox libhardware
```

## Build B2S

Either set the corresponding `b2g` key in `.servobuild` to the path to the B2G clone (along with), or set the `$GONKDIR`
environment variable.

Do the same for the `ndk` and `toolchain` keys (`$ANDROID_NDK` and `$ANDROID_TOOLCHAIN` respectively)

Run `./mach build-gonk` from the root directory


## Copy the files to the Flame

To reduce the size of libmozjs.so (`ports/gonk/target/arm-linux-androideabi/build/mozjs-sys-*/out/libmozjs.so`),
you can run `strip` on it. Use the one in your toolchain (`$ANDROID_TOOLCHAIN/bin/arm-linux-androideabi-strip libmozjs.so`).

Make sure the device is on, connected to wifi, with high or no screen timeout.

```
# Switch to a read-write system
adb remount

# Copy mozjs
adb push /path/to/stripped/mozjs.so system/lib

# Copy b2s
adb push ports/gonk/target/arm-linux-androideabi system/bin

# Copy resources
adb shell mkdir sdcard/servo
adb push resources sdcard/servo
```


## Run B2S

Make sure you're still connected to wifi

```
adb shell stop b2g
adb shell "echo 127 > /sys/class/leds/lcd-backlight/brightness”
adb shell start b2g
```

Now run `adb shell`, `cd` to `system/bin`, and run `./b2s <url>`

If the screen keeps alternating between B2G and B2S, run `adb shell stop b2g` (you can restart it later).


