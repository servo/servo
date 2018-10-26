/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package org.mozilla.servoview;

import android.app.Activity;
import android.content.res.AssetManager;
import android.content.Context;
import android.util.Log;

import java.io.IOException;
import java.io.InputStream;

import org.freedesktop.gstreamer.GStreamer;
import org.mozilla.servoview.JNIServo.ServoOptions;

public class Servo {
    private static final String LOGTAG = "Servo";
    private AssetManager mAssetMgr;
    private JNIServo mJNI = new JNIServo();
    private RunCallback mRunCallback;
    private boolean mSuspended;

    public Servo(
            ServoOptions options,
            RunCallback runCallback,
            GfxCallbacks gfxcb,
            Client client,
            Activity activity) {

        mRunCallback = runCallback;

        mAssetMgr = activity.getResources().getAssets();

        Callbacks cbs = new Callbacks(client, gfxcb);

        mRunCallback.inGLThread(() -> {
            mJNI.init(activity, options, cbs);
        });

        try {
          GStreamer.init((Context) activity);
        } catch (Exception e) {
          e.printStackTrace();
        }
    }

    public void requestShutdown() {
        mRunCallback.inGLThread(() -> mJNI.requestShutdown());
    }

    public void deinit() {
        mRunCallback.inGLThread(() -> mJNI.deinit());
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

    public void resize(int width, int height) {
        mRunCallback.inGLThread(() -> mJNI.resize(width, height));
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

        void finalizeShutdown();
    }

    public interface GfxCallbacks {
        void flushGLBuffers();

        void animationStateChanged(boolean animating);

        void makeCurrent();
    }

    private class Callbacks implements JNIServo.Callbacks, Client {

        private final GfxCallbacks mGfxCb;
        Client mClient;

        Callbacks(Client client, GfxCallbacks gfxcb) {
            mClient = client;
            mGfxCb = gfxcb;
        }

        public void wakeup() {
            if (!mSuspended) {
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
            mRunCallback.finalizeShutdown();
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

        public byte[] readfile(String file) {
            try {
                InputStream stream = mAssetMgr.open(file);
                byte[] bytes = new byte[stream.available()];
                stream.read(bytes);
                stream.close();
                return bytes;
            } catch (IOException e) {
                Log.e(LOGTAG, "readfile error: " + e.getMessage());
                return null;
            }
        }
    }
}
