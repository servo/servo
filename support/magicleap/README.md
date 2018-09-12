# Servo for Magic Leap

## Build requirements

Currently, we only support building Servo for the Magic Leap on macOS.

Install the Magic Leap Lumin and Lumin Runtime SDKs.

Get a signing key for the magic leap app.

Optionally, install Visual Studio Code and the Magic Leap plugin.

## Building the mlservo library

Build the mlservo library:
```
MAGICLEAP_SDK=*directory*  ./mach build -d --magicleap
```
This builds a static library `target/aarch64-linux-android/debug/libmlservo.a`.

## Building the Servo2D application

From inside the `support/magicleap/Servo2D` directory:
```
mabu Servo2D.package -t device -s *signing key*
```
This builds the application `.out/Servo2D/Servo2D.mpk`.

Alternatively, in Visual Studio code, open the `support/magicleap/Servo2D` directory,
and use the `Terminal/Run Build Task...` menu option to build the
Servo2D application.
