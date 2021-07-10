# A GStreamer plugin which runs servo

## Supported platforms

* MacOS + CGL
* Linux + Wayland

## Build

```
./mach build -r -p servo-gst-plugin
```

## Install

By default, gstreamer's plugin finder will complain about any libraries it finds that aren't
gstreamer plugins, so we need to have a directory just for plugins:
```
mkdir target/gstplugins
```

To install:
```
cp target/release/libgstservoplugin.* target/gstplugins
```
## Run

To run locally:
```
GST_PLUGIN_PATH=target/gstplugins \
  gst-launch-1.0 -e servowebsrc \
    ! video/x-raw\(memory:GLMemory\),framerate=50/1,width=1920,height=1080,format=RGBA \
    ! glimagesink rotate-method=vertical-flip
```

To stream over the network:
```
GST_PLUGIN_PATH=target/gstplugins \
  gst-launch-1.0 -e servowebsrc \
    ! video/x-raw\(memory:GLMemory\),framerate=50/1,width=512,height=256 \
    ! glvideoflip video-direction=vert \
    ! glcolorconvert \
    ! gldownload \
    ! theoraenc \
    ! oggmux \
    ! tcpserversink host=127.0.0.1 port=8080
```

To stream to youtube live, first go to youtube studio and create a new live stream, with its token, then:
```
GST_PLUGIN_PATH=target/gstplugins \
  gst-launch-1.0 -e servowebsrc \
    ! video/x-raw\(memory:GLMemory\),framerate=50/1,width=1960,height=1080 \
    ! glvideoflip video-direction=vert \
    ! glcolorconvert \
    ! gldownload \
    ! x264enc bitrate=6000 \
    ! flvmux name=mux \
    ! rtmpsink location="rtmp://a.rtmp.youtube.com/live2/x/$TOKEN" \
    audiotestsrc wave=silence \
    ! voaacenc bitrate=128000 \
    ! mux.
```

To  save to a file:
```
GST_PLUGIN_PATH=target/gstplugins \
  gst-launch-1.0 -e servowebsrc \
    ! video/x-raw\(memory:GLMemory\),framerate=50/1,width=512,height=256 \
    ! glvideoflip video-direction=vert \
    ! glcolorconvert \
    ! gldownload \
    ! theoraenc \
    ! oggmux \
    ! filesink location=test.ogg
```

To stream webxr content and save to a file:
```
GST_PLUGIN_PATH=target/gstplugins \
 gst-launch-1.0 -e servowebsrc url=... webxr=left-right \
   ! video/x-raw\(memory:GLMemory\),framerate=50/1,width=512,height=512,format=RGBA \
   ! glvideoflip video-direction=vert \
   ! glcolorconvert \
   ! gldownload \
   ! queue \
   ! x264enc \
   ! mp4mux \
   ! filesink location=test.mp4
```
This requires the webxr content to support the `sessionavailable` event for launching directly into immersive mode.
Values for `webxr` include `none`, `left-right`, `red-cyan`, `cubemap` and `spherical`.

To stream a Hubs room to twitch (there'll be ~30s black at the beginning while Hubs starts up):
```
GST_PLUGIN_PATH=$PWD/target/gstplugins \
  gst-launch-1.0 -e servowebsrc \
      url="https://hubs.mozilla.com/$ROOM?no_force_webvr&vr_entry_type=vr_now" \
      webxr=red-cyan \
      prefs='{"dom.gamepad.enabled":true, "dom.svg.enabled":true, "dom.canvas_capture.enabled": true, "dom.canvas_capture.enabled":true, "dom.webrtc.enabled":true, "dom.webrtc.transceiver.enabled":true}' \
    ! video/x-raw\(memory:GLMemory\),framerate=50/1,width=1920,height=1080,format=RGBA \
    ! glvideoflip video-direction=vert \
    ! glcolorconvert \
    ! gldownload \
    ! queue \
    ! x264enc bitrate=4500 \
    ! video/x-h264, profile=high \
    ! flvmux streamable=true \
    ! rtmpsink location="rtmp://$ENDPOINT.twitch.tv/app/$STREAM_KEY"
```

*Note*: killing the gstreamer pipeline with control-C sometimes locks up macOS to the point
of needing a power cycle. Killing the pipeline by closing the window seems to work.

## Troubleshooting building the plugin

You may need to make sure rust picks up the right gstreamer, for example:
```
PKG_CONFIG_PATH=$PWD/support/linux/gstreamer/gst/lib \
LD_LIBRARY_PATH=$PWD/support/linux/gstreamer/gst/lib \
  ./mach build -r -p servo-gst-plugin
```

## Troubleshooting running the plugin

First try:
```
GST_PLUGIN_PATH=target/gstplugins \
  gst-inspect-1.0 servowebsrc
```

If that doesn't work, try:
```
GST_PLUGIN_PATH=target/gstplugins \
  gst-inspect-1.0 target/gstplugins/libgstservoplugin.so
```

If you get reports about the plugin being blacklisted, remove the (global!) gstreamer cache, e.g. under Linux:
```
rm -r ~/.cache/gstreamer-1.0
```

If you get complaints about not being able to find libraries, set `LD_LIBRARY_PATH`, e.g. to use Servo's Linux gstreamer:
```
LD_LIBRARY_PATH=$PWD/support/linux/gstreamer/gst/lib
```

If you get complaints `cannot allocate memory in static TLS block` this is caused by gstreamer initializing threads using
the system alloc, which causes problems if those threads run Rust code that uses jemalloc. The fix is to preload the plugin:
```
LD_PRELOAD=$PWD/target/gstplugins/libgstservoplugin.so
```

You may need to set `GST_PLUGIN_SCANNER`, e.g. to use Servo's:
```
GST_PLUGIN_SCANNER=$PWD/support/linux/gstreamer/gst/libexec/gstreamer-1.0/gst-plugin-scanner
```

You may need to include other directories on the plugin search path, e.g. Servo's gstreamer:
```
GST_PLUGIN_PATH=$PWD/target/gstplugins/:$PWD/support/linux/gstreamer/gst/lib
```

If you get complaints `could not get/set settings from/on resource.` right after finding a GL context then try `tee`-ing to `glimagesink` and `gldownload`:
```
LD_PRELOAD=$PWD/target/gstplugins/libgstservoplugin.so \
GST_PLUGIN_PATH=target/gstplugins \
  gst-launch-1.0 servowebsrc \
    ! video/x-raw\(memory:GLMemory\),framerate=50/1,width=512,height=256 \
    !  tee name=t \
 t. ! queue ! glimagesink 
 t. ! queue ! glcolorconvert ! gldownload ! theoraenc ! oggmux ! filesink location=test.ogg
```


Under X11 you may get complaints about X11 threads not being initialized:
```
GST_GL_XINITTHREADS=1
```

Under x11 you may get a frozen display from `autovideosink`, try `ximagesink` instead.

Putting that all together:
```
GST_GL_XINITTHREADS=1 \
GST_PLUGIN_PATH=$PWD/target/gstplugins/:$PWD/support/linux/gstreamer/gst/lib \
GST_PLUGIN_SCANNER=$PWD/support/linux/gstreamer/gst/libexec/gstreamer-1.0/gst-plugin-scanner \
LD_LIBRARY_PATH=$PWD/support/linux/gstreamer/gst/lib \
LD_PRELOAD=$PWD/target/gstplugins/libgstservoplugin.so \
  gst-launch-1.0 servowebsrc \
    ! videoflip video-direction=vert \
    ! ximagesink
```
