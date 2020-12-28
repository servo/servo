# libsimpleservo
This is a basic wrapper around Servo. While libservo itself (/components/servo/) offers a lot of flexibility,
libsimpleservo (/ports/libsimpleservo/) tries to make it easier to embed Servo, without much configuration needed.
It is limited to only one view (no tabs, no multiple rendering area).

## Building
Run the following command to generate `libsimpleservo`
```
./mach build --release --libsimpleservo
```
this will generate a shared library (`libsimpleservo.so` on linux) as well as a header file in `target/release` that you can then link to your application.
