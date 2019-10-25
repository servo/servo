/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

package org.mozilla.servoview;

import android.app.Activity;
import android.net.Uri;
import android.opengl.EGL14;
import android.opengl.EGLConfig;
import android.opengl.EGLContext;
import android.opengl.EGLDisplay;
import android.opengl.EGLSurface;
import android.opengl.GLUtils;
import android.os.Handler;
import android.os.Looper;
import android.util.Log;
import android.view.Surface;

import org.mozilla.servoview.JNIServo.ServoCoordinates;
import org.mozilla.servoview.JNIServo.ServoOptions;
import org.mozilla.servoview.Servo.Client;
import org.mozilla.servoview.Servo.GfxCallbacks;
import org.mozilla.servoview.Servo.RunCallback;

import static android.opengl.EGL14.EGL_CONTEXT_CLIENT_VERSION;
import static android.opengl.EGL14.EGL_NO_CONTEXT;
import static android.opengl.EGL14.EGL_NO_SURFACE;
import static android.opengl.EGL14.EGL_OPENGL_ES2_BIT;

public class ServoSurface {
    private static final String LOGTAG = "ServoSurface";
    private final GLThread mGLThread;
    private final Handler mMainLooperHandler;
    private Handler mGLLooperHandler;
    private Surface mASurface;
    private int mPadding;
    private int mWidth;
    private int mHeight;
    private long mVRExternalContext;
    private Servo mServo;
    private Client mClient = null;
    private String mServoArgs;
    private String mServoLog;
    private String mInitialUri;
    private Activity mActivity;

    public ServoSurface(Surface surface, int width, int height, int padding) {
        mPadding = padding;
        mWidth = width;
        mHeight = height;
        mASurface = surface;
        mMainLooperHandler = new Handler(Looper.getMainLooper());
        mGLThread = new GLThread();
    }

    public void onSurfaceChanged(Surface surface) {
      mASurface = surface;
      mGLThread.onSurfaceChanged();
    }

    public void setClient(Client client) {
        mClient = client;
    }

    public void setServoArgs(String args, String log) {
        mServoArgs = args;
        mServoLog = log;
    }

    public void setActivity(Activity activity) {
        mActivity = activity;
    }

    public void setVRExternalContext(long context) {
        mVRExternalContext = context;
    }

    public void runLoop() {
        mGLThread.start();
    }

    public void shutdown() {
        Log.d(LOGTAG, "shutdown");
        mServo.shutdown();
        mServo = null;
        mGLThread.shutdown();
        try {
            Log.d(LOGTAG, "Waiting for GL thread to shutdown");
            mGLThread.join();
        } catch (InterruptedException e) {
            e.printStackTrace();
        }
    }

    public void reload() {
        mServo.reload();
    }

    public void goBack() {
        mServo.goBack();
    }

    public void goForward() {
        mServo.goForward();
    }

    public void stop() {
        mServo.stop();
    }

    public void loadUri(String uri) {
        if (mServo != null) {
            mServo.loadUri(uri);
        } else {
            mInitialUri = uri;
        }
    }

    public void loadUri(Uri uri) {
      loadUri(uri.toString());
    }

    public void scrollStart(int dx, int dy, int x, int y) {
        mServo.scrollStart(dx, dy, x, y);
    }

    public void scroll(int dx, int dy, int x, int y) {
        mServo.scroll(dx, dy, x, y);
    }

    public void scrollEnd(int dx, int dy, int x, int y) {
        mServo.scrollEnd(dx, dy, x, y);
    }

    public void click(float x, float y) {
        mServo.click(x, y);
    }

    public void onSurfaceResized(int width, int height) {
        mWidth = width;
        mHeight = height;

        ServoCoordinates coords = new ServoCoordinates();
        coords.x = mPadding;
        coords.y = mPadding;
        coords.width = width - 2 * mPadding;
        coords.height = height - 2 * mPadding;
        coords.fb_width = width;
        coords.fb_height = height;

        mServo.resize(coords);
    }

    static class GLSurface implements GfxCallbacks {
        private EGLConfig[] mEGLConfigs;
        private EGLDisplay mEglDisplay;
        private EGLContext mEglContext;
        private EGLSurface mEglSurface;
        
        void throwGLError(String function) {
            throwGLError(function, EGL14.eglGetError());
        }

        void throwGLError(String function, int error) {
            throw new RuntimeException("Error: " + function + "() Failed " + GLUtils.getEGLErrorString(error));
        }

        GLSurface(Surface surface) {
            mEglDisplay = EGL14.eglGetDisplay(EGL14.EGL_DEFAULT_DISPLAY);
            int[] version = new int[2];
            if (!EGL14.eglInitialize(mEglDisplay, version, 0, version, 1)) {
                throwGLError("eglInitialize");
            }
            mEGLConfigs = new EGLConfig[1];
            int[] configsCount = new int[1];
            int[] configSpec = new int[]{
                    EGL14.EGL_RENDERABLE_TYPE, EGL_OPENGL_ES2_BIT,
                    EGL14.EGL_RED_SIZE, 8,
                    EGL14.EGL_GREEN_SIZE, 8,
                    EGL14.EGL_BLUE_SIZE, 8,
                    EGL14.EGL_ALPHA_SIZE, 8,
                    EGL14.EGL_DEPTH_SIZE, 24,
                    EGL14.EGL_STENCIL_SIZE, 0,
                    EGL14.EGL_NONE
            };
            if ((!EGL14.eglChooseConfig(mEglDisplay, configSpec, 0, mEGLConfigs, 0, 1, configsCount, 0)) || (configsCount[0] == 0)) {
                throwGLError("eglChooseConfig");
            }
            if (mEGLConfigs[0] == null) {
                throw new RuntimeException("Error: eglConfig() not Initialized");
            }
            int[] attrib_list = {EGL_CONTEXT_CLIENT_VERSION, 3, EGL14.EGL_NONE};
            mEglContext = EGL14.eglCreateContext(mEglDisplay, mEGLConfigs[0], EGL14.EGL_NO_CONTEXT, attrib_list, 0);
            int glError = EGL14.eglGetError();
            if (glError != EGL14.EGL_SUCCESS) {
                throwGLError("eglCreateContext", glError);
            }
            mEglSurface = EGL14.eglCreateWindowSurface(mEglDisplay, mEGLConfigs[0], surface, new int[]{EGL14.EGL_NONE}, 0);
            if (mEglSurface == null || mEglSurface == EGL14.EGL_NO_SURFACE) {
                glError = EGL14.eglGetError();
                if (glError == EGL14.EGL_BAD_NATIVE_WINDOW) {
                    Log.e(LOGTAG, "Error: createWindowSurface() Returned EGL_BAD_NATIVE_WINDOW.");
                    return;
                }
                throwGLError("createWindowSurface", glError);
            }

            makeCurrent();
        }


        public void makeCurrent() {
            if (!EGL14.eglMakeCurrent(mEglDisplay, mEglSurface, mEglSurface, mEglContext)) {
                throwGLError("eglMakeCurrent");
            }
        }

        public void flushGLBuffers() {
            EGL14.eglSwapBuffers(mEglDisplay, mEglSurface);
        }

        public void animationStateChanged(boolean animating) {
            // FIXME
        }

        void destroy() {
            Log.d(LOGTAG, "Destroying surface");
            if (!EGL14.eglMakeCurrent(mEglDisplay, EGL_NO_SURFACE, EGL_NO_SURFACE, EGL_NO_CONTEXT)) {
                throwGLError("eglMakeCurrent");
            }
            if (!EGL14.eglDestroyContext(mEglDisplay, mEglContext)) {
                throwGLError("eglDestroyContext");
            }
            if (!EGL14.eglDestroySurface(mEglDisplay, mEglSurface)) {
                throwGLError("eglDestroySurface");
            }
            if (!EGL14.eglTerminate(mEglDisplay)) {
                throwGLError("eglTerminate");
            }
        }

    }

    class GLThread extends Thread implements RunCallback {
        private GLSurface mSurface;

        public void inGLThread(Runnable r) {
            mGLLooperHandler.post(r);
        }

        public void inUIThread(Runnable r) {
            mMainLooperHandler.post(r);
        }

        public void onSurfaceChanged() {
          Log.d(LOGTAG, "GLThread::onSurfaceChanged");
          mSurface.destroy();
          mSurface = new GLSurface(mASurface);
          mServo.resetGfxCallbacks(mSurface);
        }

        public void shutdown() {
            Log.d(LOGTAG, "GLThread::shutdown");
            mSurface.destroy();
            mGLLooperHandler.getLooper().quitSafely();
        }

        public void run() {
            Looper.prepare();

            mSurface = new GLSurface(mASurface);

            mGLLooperHandler = new Handler();

            inUIThread(() -> {
              ServoCoordinates coords = new ServoCoordinates();
              coords.x = mPadding;
              coords.y = mPadding;
              coords.width = mWidth - 2 * mPadding;
              coords.height = mHeight - 2 * mPadding;
              coords.fb_width = mWidth;
              coords.fb_height = mHeight;

              ServoOptions options = new ServoOptions();
              options.coordinates = coords;
              options.args = mServoArgs;
              options.density = 1;
              options.url = mInitialUri;
              options.logStr = mServoLog;
              options.enableLogs = true;
              options.enableSubpixelTextAntialiasing = false;
              options.VRExternalContext = mVRExternalContext;

              mServo = new Servo(options, this, mSurface, mClient, mActivity);
            });

            Looper.loop();
        }
    }
}
