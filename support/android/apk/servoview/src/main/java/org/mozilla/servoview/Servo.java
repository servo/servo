/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

package org.mozilla.servoview;

import android.app.Activity;
import android.content.Context;
import android.util.Log;

import java.util.concurrent.Callable;
import java.util.concurrent.FutureTask;

import org.freedesktop.gstreamer.GStreamer;
import org.mozilla.servoview.JNIServo.ServoCoordinates;
import org.mozilla.servoview.JNIServo.ServoOptions;

public class Servo {
    private static final String LOGTAG = "Servo";
    private JNIServo mJNI = new JNIServo();
    private RunCallback mRunCallback;
    private boolean mShuttingDown;
    private boolean mShutdownComplete;
    private boolean mSuspended;
    private Callbacks mServoCallbacks;

    public Servo(
            ServoOptions options,
            RunCallback runCallback,
            GfxCallbacks gfxcb,
            Client client,
            Activity activity) {

        mRunCallback = runCallback;

        mServoCallbacks = new Callbacks(client, gfxcb);

        mRunCallback.inGLThread(() -> {
            mJNI.init(activity, options, mServoCallbacks);
        });

        try {
          GStreamer.init((Context) activity);
        } catch (Exception e) {
          e.printStackTrace();
        }
    }

    public void resetGfxCallbacks(GfxCallbacks gfxcb) {
      mServoCallbacks.resetGfxCallbacks(gfxcb);
    }

    public void shutdown() {
        mShuttingDown = true;
        FutureTask<Void> task = new FutureTask<Void>(new Callable<Void>() {
            public Void call() throws Exception {
                mJNI.requestShutdown();
                // Wait until Servo gets back to us to finalize shutdown.
                while (!mShutdownComplete) {
                    try {
                        Thread.sleep(10);
                    } catch (Exception e) {
                        mShutdownComplete = true;
                        e.printStackTrace();
                        return null;
                    }
                    mJNI.performUpdates();
                }
                mJNI.deinit();
                return null;
            }
        });
        mRunCallback.inGLThread(task);
        // Block until task is complete.
        try {
            task.get();
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    public String version() {
        return mJNI.version();
    }

    public void performUpdates() {
        mRunCallback.inGLThread(() -> mJNI.performUpdates());
    }

    public void setBatchMode(boolean mode) {
        mRunCallback.inGLThread(() -> mJNI.setBatchMode(mode));
    }

    public void resize(ServoCoordinates coords) {
        mRunCallback.inGLThread(() -> mJNI.resize(coords));
    }

    public void refresh() {
        mRunCallback.inGLThread(() -> mJNI.refresh());
    }

    public void reload() {
        mRunCallback.inGLThread(() -> mJNI.reload());
    }

    public void stop() {
        mRunCallback.inGLThread(() -> mJNI.stop());
    }

    public void goBack() {
        mRunCallback.inGLThread(() -> mJNI.goBack());
    }

    public void goForward() {
        mRunCallback.inGLThread(() -> mJNI.goForward());
    }

    public void loadUri(String uri) {
        mRunCallback.inGLThread(() -> mJNI.loadUri(uri));
    }

    public void scrollStart(int dx, int dy, int x, int y) {
        mRunCallback.inGLThread(() -> mJNI.scrollStart(dx, dy, x, y));
    }

    public void scroll(int dx, int dy, int x, int y) {
        mRunCallback.inGLThread(() -> mJNI.scroll(dx, dy, x, y));
    }

    public void scrollEnd(int dx, int dy, int x, int y) {
        mRunCallback.inGLThread(() -> mJNI.scrollEnd(dx, dy, x, y));
    }

    public void touchDown(float x, float y, int pointerId) {
        mRunCallback.inGLThread(() -> mJNI.touchDown(x, y, pointerId));
    }

    public void touchMove(float x, float y, int pointerId) {
        mRunCallback.inGLThread(() -> mJNI.touchMove(x, y, pointerId));
    }

    public void touchUp(float x, float y, int pointerId) {
        mRunCallback.inGLThread(() -> mJNI.touchUp(x, y, pointerId));
    }

    public void touchCancel(float x, float y, int pointerId) {
        mRunCallback.inGLThread(() -> mJNI.touchCancel(x, y, pointerId));
    }

    public void pinchZoomStart(float factor, int x, int y) {
        mRunCallback.inGLThread(() -> mJNI.pinchZoomStart(factor, x, y));
    }

    public void pinchZoom(float factor, int x, int y) {
        mRunCallback.inGLThread(() -> mJNI.pinchZoom(factor, x, y));
    }

    public void pinchZoomEnd(float factor, int x, int y) {
        mRunCallback.inGLThread(() -> mJNI.pinchZoomEnd(factor, x, y));
    }

    public void click(int x, int y) {
        mRunCallback.inGLThread(() -> mJNI.click(x, y));
    }

    public void suspend(boolean suspended) {
        mSuspended = suspended;
    }

    public interface Client {
        void onLoadStarted();

        void onLoadEnded();

        void onTitleChanged(String title);

        void onUrlChanged(String url);

        void onHistoryChanged(boolean canGoBack, boolean canGoForward);

        void onRedrawing(boolean redrawing);
    }

    public interface RunCallback {
        void inGLThread(Runnable f);

        void inUIThread(Runnable f);
    }

    public interface GfxCallbacks {
        void flushGLBuffers();

        void animationStateChanged(boolean animating);

        void makeCurrent();
    }

    private class Callbacks implements JNIServo.Callbacks, Client {

        private GfxCallbacks mGfxCb;
        Client mClient;

        Callbacks(Client client, GfxCallbacks gfxcb) {
            mClient = client;
            mGfxCb = gfxcb;
        }

        private void resetGfxCallbacks(GfxCallbacks gfxcb) {
          mGfxCb = gfxcb;
        }

        public void wakeup() {
            if (!mSuspended && !mShuttingDown) {
                mRunCallback.inGLThread(() -> mJNI.performUpdates());
            }
        }

        public void flush() {
            // Up to the callback to execute this in the right thread
            mGfxCb.flushGLBuffers();
        }

        public void makeCurrent() {
            // Up to the callback to execute this in the right thread
            mGfxCb.makeCurrent();
        }

        public void onShutdownComplete() {
            mShutdownComplete = true;
        }

        public void onAnimatingChanged(boolean animating) {
            mRunCallback.inGLThread(() -> mGfxCb.animationStateChanged(animating));
        }

        public void onLoadStarted() {
            mRunCallback.inUIThread(() -> mClient.onLoadStarted());
        }

        public void onLoadEnded() {
            mRunCallback.inUIThread(() -> mClient.onLoadEnded());
        }

        public void onTitleChanged(String title) {
            mRunCallback.inUIThread(() -> mClient.onTitleChanged(title));
        }

        public void onUrlChanged(String url) {
            mRunCallback.inUIThread(() -> mClient.onUrlChanged(url));
        }

        public void onHistoryChanged(boolean canGoBack, boolean canGoForward) {
            mRunCallback.inUIThread(() -> mClient.onHistoryChanged(canGoBack, canGoForward));
        }

        public void onRedrawing(boolean redrawing) {
            mRunCallback.inUIThread(() -> mClient.onRedrawing(redrawing));
        }
    }
}
