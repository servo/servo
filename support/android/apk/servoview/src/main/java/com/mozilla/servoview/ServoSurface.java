package com.mozilla.servoview;

import android.annotation.SuppressLint;
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
import android.os.Message;
import android.util.Log;

import com.mozilla.servoview.Servo.Client;
import com.mozilla.servoview.Servo.GfxCallbacks;
import com.mozilla.servoview.Servo.RunCallback;

import static android.opengl.EGL14.EGL_CONTEXT_CLIENT_VERSION;
import static android.opengl.EGL14.EGL_OPENGL_ES2_BIT;

public class ServoSurface {
    private final GLThread mGLThread;
    private final Handler mMainLooperHandler;
    private Handler mGLLooperHandler;
    private Surface mASurface;
    private int mWidth;
    private int mHeight;
    private Servo mServo;
    private Client mClient = null;
    private String mServoArgs = "";
    private Uri mInitialUri = null;
    private Activity mActivity;

    public ServoSurface(Surface surface, int width, int height) {
        mWidth = width;
        mHeight = height;
        mASurface = surface;
        mMainLooperHandler = new Handler(Looper.getMainLooper());
        mGLThread = new GLThread();
    }

    public void setClient(Client client) {
        mClient = client;
    }

    public void setServoArgs(String args) {
        mServoArgs = args != null ? args : "";
    }

    public void setActivity(Activity activity) {
        mActivity = activity;
    }

    public void runLoop() {
        mGLThread.start();
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

    public void onSurfaceResized(int width, int height) {
        mServo.resize(width, height);
    }

    public void loadUri(Uri uri) {
        if (mServo != null) {
            mServo.loadUri(uri.toString());
        } else {
            mInitialUri = uri;
        }
    }

    static class Surface implements GfxCallbacks {
        private static final String LOGTAG = "ServoSurface";

        private EGLConfig[] mEGLConfigs;
        private EGLDisplay mEglDisplay;
        private EGLContext mEglContext;
        private EGLSurface mEglSurface;

        Surface(Surface surface) {
            mEglDisplay = EGL14.eglGetDisplay(EGL14.EGL_DEFAULT_DISPLAY);
            int[] version = new int[2];
            if (!EGL14.eglInitialize(mEglDisplay, version, 0, version, 1)) {
                throw new RuntimeException("Error: eglInitialize() Failed " + GLUtils.getEGLErrorString(EGL14.eglGetError()));
            }
            mEGLConfigs = new EGLConfig[1];
            int[] configsCount = new int[1];
            int[] configSpec = new int[]{
                    EGL14.EGL_RENDERABLE_TYPE, EGL_OPENGL_ES2_BIT,
                    EGL14.EGL_RED_SIZE, 8,
                    EGL14.EGL_GREEN_SIZE, 8,
                    EGL14.EGL_BLUE_SIZE, 8,
                    EGL14.EGL_ALPHA_SIZE, 8,
                    EGL14.EGL_DEPTH_SIZE, 0,
                    EGL14.EGL_STENCIL_SIZE, 0,
                    EGL14.EGL_NONE
            };
            if ((!EGL14.eglChooseConfig(mEglDisplay, configSpec, 0, mEGLConfigs, 0, 1, configsCount, 0)) || (configsCount[0] == 0)) {
                throw new IllegalArgumentException("Error: eglChooseConfig() Failed " + GLUtils.getEGLErrorString(EGL14.eglGetError()));
            }
            if (mEGLConfigs[0] == null) {
                throw new RuntimeException("Error: eglConfig() not Initialized");
            }
            int[] attrib_list = {EGL_CONTEXT_CLIENT_VERSION, 3, EGL14.EGL_NONE};
            mEglContext = EGL14.eglCreateContext(mEglDisplay, mEGLConfigs[0], EGL14.EGL_NO_CONTEXT, attrib_list, 0);
            int glError = EGL14.eglGetError();
            if (glError != EGL14.EGL_SUCCESS) {
                throw new RuntimeException("Error: eglCreateContext() Failed " + GLUtils.getEGLErrorString(glError));
            }
            mEglSurface = EGL14.eglCreateWindowSurface(mEglDisplay, mEGLConfigs[0], surface, new int[]{EGL14.EGL_NONE}, 0);
            if (mEglSurface == null || mEglSurface == EGL14.EGL_NO_SURFACE) {
                glError = EGL14.eglGetError();
                if (glError == EGL14.EGL_BAD_NATIVE_WINDOW) {
                    Log.e(LOGTAG, "Error: createWindowSurface() Returned EGL_BAD_NATIVE_WINDOW.");
                    return;
                }
                throw new RuntimeException("Error: createWindowSurface() Failed " + GLUtils.getEGLErrorString(glError));
            }

            flushGLBuffers();
        }


        public void makeCurrent() {
            if (!EGL14.eglMakeCurrent(mEglDisplay, mEglSurface, mEglSurface, mEglContext)) {
                throw new RuntimeException("Error: eglMakeCurrent() Failed " + GLUtils.getEGLErrorString(EGL14.eglGetError()));
            }
        }

        public void flushGLBuffers() {
            EGL14.eglSwapBuffers(mEglDisplay, mEglSurface);
        }

        public void animationStateChanged(boolean animating) {
            // FIXME
        }

    }

    class GLThread extends Thread implements RunCallback {

        public void inGLThread(Runnable r) {
            mGLLooperHandler.post(r);
        }

        public void inUIThread(Runnable r) {
            mMainLooperHandler.post(r);
        }

        // FIXME: HandlerLeak
        @SuppressLint("HandlerLeak")
        public void run() {
            Looper.prepare();

            Surface surface = new Surface(mASurface);

            final boolean showLogs = true;
            String uri = mInitialUri == null ? null : mInitialUri.toString();
            mServo = new Servo(this, surface, mClient, mActivity, mServoArgs, uri, mWidth, mHeight, showLogs);

            mGLLooperHandler = new Handler() {
                public void handleMessage(Message msg) {
                }
            };

            Looper.loop();
        }
    }
}
