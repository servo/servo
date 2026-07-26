/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

package org.servo.servoview;

import android.content.Context;
import android.view.KeyEvent;
import android.view.Surface;

import org.servo.servoview.JNIServo.ServoCoordinates;
import org.servo.servoview.JNIServo.ServoOptions;

public class Servo {
    private static final String LOGTAG = "Servo";
    private JNIServo jni = new JNIServo();
    private RunCallback runCallback;
    private boolean suspended;
    private Callbacks servoCallbacks;

    public Servo(
            ServoOptions options,
            RunCallback runCallback,
            Client client,
            Context context,
            Surface surface) {

        this.runCallback = runCallback;

        servoCallbacks = new Callbacks(client);

        this.runCallback.inGLThread(() -> jni.init(context, options, servoCallbacks, surface));
    }

    public String version() {
        return jni.version();
    }

    public void performUpdates() {
        runCallback.inGLThread(() -> jni.performUpdates());
    }

    public void resize(ServoCoordinates coords) {
        runCallback.inGLThread(() -> jni.resize(coords));
    }

    public void reload() {
        runCallback.inGLThread(() -> jni.reload());
    }

    public void stop() {
        runCallback.inGLThread(() -> jni.stop());
    }

    public void goBack() {
        runCallback.inGLThread(() -> jni.goBack());
    }

    public void goForward() {
        runCallback.inGLThread(() -> jni.goForward());
    }

    public void loadUri(String uri) {
        runCallback.inGLThread(() -> jni.loadUri(uri));
    }

    public void scroll(int dx, int dy, int x, int y) {
        runCallback.inGLThread(() -> jni.scroll(dx, dy, x, y));
    }

    public void onKeyDown(int keyCode, KeyEvent event) {
        runCallback.inGLThread(() -> jni.keydown(keyCode, event.getUnicodeChar()));
    }

    public void onKeyUp(int keyCode, KeyEvent event) {
        runCallback.inGLThread(() -> jni.keyup(keyCode, event.getUnicodeChar()));
    }

    public void touchDown(float x, float y, int pointerId) {
        runCallback.inGLThread(() -> jni.touchDown(x, y, pointerId));
    }

    public void touchMove(float x, float y, int pointerId) {
        runCallback.inGLThread(() -> jni.touchMove(x, y, pointerId));
    }

    public void touchUp(float x, float y, int pointerId) {
        runCallback.inGLThread(() -> jni.touchUp(x, y, pointerId));
    }

    public void touchCancel(float x, float y, int pointerId) {
        runCallback.inGLThread(() -> jni.touchCancel(x, y, pointerId));
    }

    public void pinchZoomStart(float factor, float x, float y) {
        runCallback.inGLThread(() -> jni.pinchZoomStart(factor, x, y));
    }

    public void pinchZoom(float factor, float x, float y) {
        runCallback.inGLThread(() -> jni.pinchZoom(factor, x, y));
    }

    public void pinchZoomEnd(float factor, float x, float y) {
        runCallback.inGLThread(() -> jni.pinchZoomEnd(factor, x, y));
    }

    public void click(float x, float y) {
        runCallback.inGLThread(() -> jni.click(x, y));
    }

    public void pausePainting() {
        runCallback.inGLThread(() -> jni.pausePainting());
    }

    public void resumePainting(Surface surface, ServoCoordinates coords) {
        runCallback.inGLThread(() -> jni.resumePainting(surface, coords));
    }

    public void suspend(boolean suspended) {
        this.suspended = suspended;
    }

    public void mediaSessionAction(int action) {
        runCallback.inGLThread(() -> jni.mediaSessionAction(action));
    }

    public void setExperimentalMode(boolean enable) {
        runCallback.inGLThread(() -> jni.setExperimentalMode(enable));
    }

    public void onDoFrame() {
        runCallback.inGLThread(() -> jni.doFrame());
    }

    public interface Client {
        void onAlert(String message);

        void onLoadStarted();

        void onLoadEnded();

        void onTitleChanged(String title);

        void onUrlChanged(String url);

        void onHistoryChanged(boolean canGoBack, boolean canGoForward);

        void onRedrawing(boolean redrawing);

        void onImeShow();

        void onImeHide();

        void onMediaSessionMetadata(String title, String artist, String album);

        void onMediaSessionPlaybackStateChange(int state);

        void onMediaSessionSetPositionState(float duration, float position, float playbackRate);
    }

    public interface RunCallback {
        void inGLThread(Runnable f);

        void inUIThread(Runnable f);
    }

    private class Callbacks implements JNIServo.Callbacks, Client {

        Client client;

        Callbacks(Client client) {
            this.client = client;
        }

        public void wakeup() {
            if (!suspended) {
                runCallback.inGLThread(() -> jni.performUpdates());
            }
        }

        public void onAlert(String message) {
            runCallback.inUIThread(() -> client.onAlert(message));
        }

        public void onImeShow() {
            runCallback.inUIThread(() -> client.onImeShow());
        }

        public void onImeHide() {
            runCallback.inUIThread(() -> client.onImeHide());
        }

        public void onLoadStarted() {
            runCallback.inUIThread(() -> client.onLoadStarted());
        }

        public void onLoadEnded() {
            runCallback.inUIThread(() -> client.onLoadEnded());
        }

        public void onTitleChanged(String title) {
            runCallback.inUIThread(() -> client.onTitleChanged(title));
        }

        public void onUrlChanged(String url) {
            runCallback.inUIThread(() -> client.onUrlChanged(url));
        }

        public void onHistoryChanged(boolean canGoBack, boolean canGoForward) {
            runCallback.inUIThread(() -> client.onHistoryChanged(canGoBack, canGoForward));
        }

        public void onRedrawing(boolean redrawing) {
            runCallback.inUIThread(() -> client.onRedrawing(redrawing));
        }

        public void onMediaSessionMetadata(String title, String artist, String album) {
            runCallback.inUIThread(() -> client.onMediaSessionMetadata(title, artist, album));
        }

        public void onMediaSessionPlaybackStateChange(int state) {
            runCallback.inUIThread(() -> client.onMediaSessionPlaybackStateChange(state));
        }

        public void onMediaSessionSetPositionState(float duration, float position, float playbackRate) {
            runCallback.inUIThread(() -> client.onMediaSessionSetPositionState(duration, position, playbackRate));
        }
    }
}
