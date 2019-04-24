/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

package org.mozilla.servoview;

import android.app.Activity;

/**
 * Maps /ports/libsimpleservo API
 */
@SuppressWarnings("JniMissingFunction")
public class JNIServo {
    JNIServo() {
        System.loadLibrary("c++_shared");
        System.loadLibrary("gstreamer_android");
        System.loadLibrary("simpleservo");
    }

    public native String version();

    public native void init(Activity activity, ServoOptions options, Callbacks callbacks);

    public native void deinit();

    public native void requestShutdown();

    public native void setBatchMode(boolean mode);

    public native void performUpdates();

    public native void resize(ServoCoordinates coords);

    public native void reload();

    public native void stop();

    public native void refresh();

    public native void goBack();

    public native void goForward();

    public native void loadUri(String uri);

    public native void scrollStart(int dx, int dy, int x, int y);

    public native void scroll(int dx, int dy, int x, int y);

    public native void scrollEnd(int dx, int dy, int x, int y);

    public native void touchDown(float x, float y, int pointer_id);

    public native void touchMove(float x, float y, int pointer_id);

    public native void touchUp(float x, float y, int pointer_id);

    public native void touchCancel(float x, float y, int pointer_id);

    public native void pinchZoomStart(float factor, int x, int y);

    public native void pinchZoom(float factor, int x, int y);

    public native void pinchZoomEnd(float factor, int x, int y);

    public native void click(int x, int y);

    public static class ServoOptions {
      public String args;
      public String url;
      public ServoCoordinates coordinates;
      public float density = 1;
      public boolean enableSubpixelTextAntialiasing = true;
      public long VRExternalContext = 0;
      public String logStr;
      public boolean enableLogs = false;
    }

    public static class ServoCoordinates {
      public int x = 0;
      public int y = 0;
      public int width = 0;
      public int height = 0;
      public int fb_width = 0;
      public int fb_height = 0;
    }

    public interface Callbacks {
        void wakeup();

        void flush();

        void makeCurrent();

        void onAnimatingChanged(boolean animating);

        void onLoadStarted();

        void onLoadEnded();

        void onTitleChanged(String title);

        void onUrlChanged(String url);

        void onHistoryChanged(boolean canGoBack, boolean canGoForward);

        void onShutdownComplete();
    }
}

