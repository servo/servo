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
This builds a static library `target/magicleap/aarch64-linux-android/debug/libmlservo.a`.

## Building the Servo2D application

From inside the `support/magicleap/Servo2D` directory:
```
mabu Servo2D.package -t device -s *signing key*
```
This builds the application `.out/Servo2D/Servo2D.mpk`.

Alternatively, in Visual Studio code, open the `support/magicleap/Servo2D` directory,
and use the `Terminal/Run Build Task...` menu option to build the
Servo2D application.

## Debugging gstreamer

By default, Servo links against release builds of gstreamer, even for debug builds,
so if you want to use gdb on gstreamer, you've got some work to do...

First off, you'll need to build the gstreamer libraries with debug symbols.
To do this, edit `support/magicleap/gstreamer/mlsdk.txt.in` and add `'-g', '-O0`,` to
`c_args` and `cpp_args`:

```
[properties]
c_args = [
  '-g', '-O0',
  '--sysroot=@MAGICLEAP_SDK@/lumin/usr',
  '-I@MAGICLEAP_SDK@/include',
  '-I@MAGICLEAP_SDK@/staging/include',
  '-I@INSTALL_DIR@/system/include',
  ]
cpp_args = [
  '-g', '-O0',
  '--sysroot=@MAGICLEAP_SDK@/lumin/usr',
  '-I@MAGICLEAP_SDK@/include',
  '-I@MAGICLEAP_SDK@/staging/include',
  '-I@INSTALL_DIR@/system/include',
  ]
```
then build the libraries with `./gstreamer.sh` from inside `support/magicleap/gstreamer`.

The libraries will be built in `_install/system`, and should be moved over to
where `mach` expects them to be:

```
rm -r target/magicleap/aarch64-linux-android/native/gstreamer-1.16.0/system
cp -r support/magicleap/gstreamer/_install/system target/magicleap/aarch64-linux-android/native/gstreamer-1.16.0/
```
You can now build, package and install as normal:
```
./mach build -d --magicleap
./mach package -d --magicleap
$MAGICLEAP_SDK/tools/mldb/mldb install -u target/magicleap/aarch64-linux-android/debug/Servo.mpk
```
to launch gdb on the application:
```
$MAGICLEAP_SDK/debug --package com.mozilla.servo support/magicleap/Servo2D/.out/debug_lumin_clang-3.8_aarch64/Servo2D
```
Using the debug libraries in gdb is slightly tricky because you need to set everything up
in the right order, setting `solib-search-path` should happen before doing any dynamic loading,
but setting `sysroot` after dynamic loading has started. The easiest thing to do is to place a
breakpoint somewhere in Servo after gstreamer has started loading, for example:


```
(gdb) set solib-search-path /Users/ajeffrey/MagicLeap/mlsdk/v0.22.0/lumin/usr/lib:/Users/ajeffrey/github/asajeffrey/servo/target/magicleap/aarch64-linux-android/native/gstreamer-1.16.0/system/lib64/
warning: Unable to find dynamic linker breakpoint function.
GDB will be unable to debug shared library initializers
and track explicitly loaded dynamic code.

(gdb) rbreak RenderAndroid::new
Breakpoint 1 at 0xaaaac59953a4
struct Option<servo_media_gstreamer_render_android::RenderAndroid> servo_media_gstreamer_render_android::RenderAndroid::new::he5ddafe679ae0dff(struct Box<PlayerGLContext>);
Breakpoint 2 at 0xaaaac423f184: file /Users/ajeffrey/github/asajeffrey/media/backends/gstreamer/render-android/lib.rs, line 81.
static struct Option<gstreamer_gl::auto::gl_display::GLDisplay> servo_media_gstreamer_render_android::RenderAndroid::new::_$u7b$$u7b$closure$u7d$$u7d$::h53b9e99990e38d92(struct closure, struct GLDisplayEGL);

(gdb) c

...
Thread 2 "ScriptThread Pi" hit Breakpoint 1, 0x0000aaaac59953a4 in servo_media_gstreamer_render_android::RenderAndroid::new::he5ddafe679ae0dff
    (app_gl_context=...)
(gdb) set sysroot /Users/ajeffrey/MagicLeap/mlsdk/v0.22.0/lumin
warning: .dynamic section for "/Users/ajeffrey/MagicLeap/mlsdk/v0.22.0/lumin/usr/lib/libc.so" is not at the expected address (wrong library or version mismatch?)
warning: .dynamic section for "/Users/ajeffrey/MagicLeap/mlsdk/v0.22.0/lumin/usr/lib/liblog.so" is not at the expected address (wrong library or version mismatch?)
warning: .dynamic section for "/Users/ajeffrey/MagicLeap/mlsdk/v0.22.0/lumin/usr/lib/libz.so" is not at the expected address (wrong library or version mismatch?)
warning: .dynamic section for "/Users/ajeffrey/MagicLeap/mlsdk/v0.22.0/lumin/usr/lib/libEGL.so" is not at the expected address (wrong library or version mismatch?)
warning: .dynamic section for "/Users/ajeffrey/MagicLeap/mlsdk/v0.22.0/lumin/usr/lib/libGLESv2.so" is not at the expected address (wrong library or version mismatch?)
warning: .dynamic section for "/Users/ajeffrey/MagicLeap/mlsdk/v0.22.0/lumin/usr/lib/libGLESv3.so" is not at the expected address (wrong library or version mismatch?)
warning: .dynamic section for "/Users/ajeffrey/MagicLeap/mlsdk/v0.22.0/lumin/usr/lib/libm.so" is not at the expected address (wrong library or version mismatch?)
warning: .dynamic section for "/Users/ajeffrey/MagicLeap/mlsdk/v0.22.0/lumin/usr/lib/libvulkan.so" is not at the expected address (wrong library or version mismatch?)
warning: .dynamic section for "/Users/ajeffrey/MagicLeap/mlsdk/v0.22.0/lumin/usr/lib/libstdc++.so" is not at the expected address (wrong library or version mismatch?)
warning: .dynamic section for "/Users/ajeffrey/MagicLeap/mlsdk/v0.22.0/lumin/usr/lib/libGLESv1_CM.so" is not at the expected address (wrong library or version mismatch?)
warning: Could not load shared library symbols for 191 libraries, e.g. /system/bin/linker64.
Use the "info sharedlibrary" command to see the complete listing.
Do you need "set solib-search-path" or "set sysroot"?
Reading symbols from /Users/ajeffrey/MagicLeap/mlsdk/v0.22.0/lumin/usr/lib/libc.so...done.
Reading symbols from /Users/ajeffrey/github/asajeffrey/servo/target/magicleap/aarch64-linux-android/native/gstreamer-1.16.0/system/lib64/libglib-2.0.so...done.
Reading symbols from /Users/ajeffrey/github/asajeffrey/servo/target/magicleap/aarch64-linux-android/native/gstreamer-1.16.0/system/lib64/libgobject-2.0.so...done.
Reading symbols from /Users/ajeffrey/github/asajeffrey/servo/target/magicleap/aarch64-linux-android/native/gstreamer-1.16.0/system/lib64/libgstreamer-1.0.so...done.
Reading symbols from /Users/ajeffrey/github/asajeffrey/servo/target/magicleap/aarch64-linux-android/native/gstreamer-1.16.0/system/lib64/libgstapp-1.0.so...done.
Reading symbols from /Users/ajeffrey/github/asajeffrey/servo/target/magicleap/aarch64-linux-android/native/gstreamer-1.16.0/system/lib64/libgstaudio-1.0.so...done.
Reading symbols from /Users/ajeffrey/github/asajeffrey/servo/target/magicleap/aarch64-linux-android/native/gstreamer-1.16.0/system/lib64/libgstbase-1.0.so...done.
Reading symbols from /Users/ajeffrey/github/asajeffrey/servo/target/magicleap/aarch64-linux-android/native/gstreamer-1.16.0/system/lib64/libgstgl-1.0.so...done.
Reading symbols from /Users/ajeffrey/github/asajeffrey/servo/target/magicleap/aarch64-linux-android/native/gstreamer-1.16.0/system/lib64/libgstplayer-1.0.so...done.
Reading symbols from /Users/ajeffrey/github/asajeffrey/servo/target/magicleap/aarch64-linux-android/native/gstreamer-1.16.0/system/lib64/libgstsdp-1.0.so...done.
Reading symbols from /Users/ajeffrey/github/asajeffrey/servo/target/magicleap/aarch64-linux-android/native/gstreamer-1.16.0/system/lib64/libgstvideo-1.0.so...done.
Reading symbols from /Users/ajeffrey/github/asajeffrey/servo/target/magicleap/aarch64-linux-android/native/gstreamer-1.16.0/system/lib64/libgstwebrtc-1.0.so...done.
...

(gdb) break gst_gl_context_activate
Breakpoint 3 at 0x40003ba0bb98: file ../gst-build/subprojects/gst-plugins-base/gst-libs/gst/gl/gstglcontext.c, line 746.
```
At this point, setting breakpoints and step-debugging should work as expected.
