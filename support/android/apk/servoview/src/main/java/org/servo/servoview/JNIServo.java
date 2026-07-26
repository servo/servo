/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

package org.servo.servoview;

import android.content.Context;
import android.view.Surface;

/**
 * Maps /ports/servoshell API
 */
@SuppressWarnings("JniMissingFunction")
class JNIServo {
    JNIServo() {
        System.loadLibrary("c++_shared");
        System.loadLibrary("servoshell");
    }

    native String version();

    native void init(Context context, ServoOptions options, Callbacks callbacks, Surface surface);

    native void performUpdates();

    native void resize(ServoCoordinates coords);

    native void reload();

    native void stop();

    native void goBack();

    native void goForward();

    native void loadUri(String uri);

    native void scroll(int dx, int dy, int x, int y);

    native void keydown(int keycode, int unicode);

    native void keyup(int keycode, int unicode);

    native void touchDown(float x, float y, int pointer_id);

    native void touchMove(float x, float y, int pointer_id);

    native void touchUp(float x, float y, int pointer_id);

    native void touchCancel(float x, float y, int pointer_id);

    native void pinchZoomStart(float factor, float x, float y);

    native void pinchZoom(float factor, float x, float y);

    native void pinchZoomEnd(float factor, float x, float y);

    native void click(float x, float y);

    native void pausePainting();

    native void resumePainting(Surface surface, ServoCoordinates coords);

    native void mediaSessionAction(int action);

    native void setExperimentalMode(boolean enable);

    native void doFrame();

    static class ServoOptions {
        String args;
        String url;
        ServoCoordinates coordinates;
        float density = 1;
        String logStr;
        boolean enableLogs = false;
        boolean experimentalMode = false;
    }

    static class ServoCoordinates {
        int width = 0;
        int height = 0;
    }

    interface Callbacks {
        void wakeup();

        void onAlert(String message);

        void onLoadStarted();

        void onLoadEnded();

        void onTitleChanged(String title);

        void onUrlChanged(String url);

        void onHistoryChanged(boolean canGoBack, boolean canGoForward);

        void onImeShow();

        void onImeHide();

        void onMediaSessionMetadata(String title, String artist, String album);

        void onMediaSessionPlaybackStateChange(int state);

        void onMediaSessionSetPositionState(float duration, float position, float playbackRate);
    }
}
