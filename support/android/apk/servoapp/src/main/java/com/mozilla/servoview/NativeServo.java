/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.servoview;

import android.app.Activity;

/**
 * Maps /ports/libsimpleservo API
 */
public class NativeServo {
    public native String version();
    public native void init(Activity activity,
                            String args,
                            String url,
                            WakeupCallback wakeup,
                            ReadFileCallback readfile,
                            ServoCallbacks callbacks,
                            int width, int height, boolean log);
    public native void setBatchMode(boolean mode);
    public native void performUpdates();
    public native void resize(int width, int height);
    public native void reload();
    public native void stop();
    public native void goBack();
    public native void goForward();
    public native void loadUri(String uri);
    public native void scrollStart(int dx, int dy, int x, int y);
    public native void scroll(int dx, int dy, int x, int y);
    public native void scrollEnd(int dx, int dy, int x, int y);
    public native void click(int x, int y);

    NativeServo() {
        System.loadLibrary("c++_shared");
        System.loadLibrary("simpleservo");
    }

    public interface ReadFileCallback {
        byte[] readfile(String file);
    }

    public interface WakeupCallback {
        void wakeup();
    }

    public interface ServoCallbacks {
        void flush();
        void onLoadStarted();
        void onLoadEnded();
        void onTitleChanged(String title);
        void onUrlChanged(String url);
        void onHistoryChanged(boolean canGoBack, boolean canGoForward);
        void onAnimatingChanged(boolean animating);
    }
}
