package org.freedesktop.gstreamer;

import android.content.Context;

public class GStreamer {
    private static native void nativeInit(Context context) throws Exception;

    public static void init(Context context) throws Exception {
        nativeInit(context);
    }
}
