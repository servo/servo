/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

package org.servo.servoview;

import android.app.Activity;
import android.content.Context;
import android.util.AttributeSet;
import android.net.Uri;
import android.os.Handler;
import android.os.Looper;
import android.util.DisplayMetrics;
import android.util.Log;
import android.view.KeyEvent;
import android.view.Surface;
import android.view.SurfaceView;
import android.view.SurfaceHolder;

import org.servo.servoview.JNIServo.ServoCoordinates;
import org.servo.servoview.JNIServo.ServoOptions;
import org.servo.servoview.Servo.Client;
import org.servo.servoview.Servo.GfxCallbacks;
import org.servo.servoview.Servo.RunCallback;

import android.view.Choreographer;
import android.view.GestureDetector;
import android.view.MotionEvent;
import android.view.ScaleGestureDetector;
import android.view.View;
import android.widget.OverScroller;

import java.util.ArrayList;

public class ServoView extends SurfaceView
                        implements
                        GfxCallbacks,
                        RunCallback,
                        Choreographer.FrameCallback {
    private static final String LOGTAG = "ServoView";
    private GLThread mGLThread;
    private Handler mGLLooperHandler;
    private Surface mASurface;
    protected Servo mServo = null;
    private Client mClient = null;
    private String mServoArgs;
    private String mServoLog;
    private String mInitialUri;
    private Activity mActivity;

    private boolean mExperimentalMode;
    private boolean mPaused = false;

    public ServoView(Context context) {
        super(context);
        init(context);
    }

    public ServoView(Context context, AttributeSet attrs) {
        super(context, attrs);
        init(context);
    }

    private void init(Context context) {
        mActivity = (Activity) context;
        setFocusable(true);
        setFocusableInTouchMode(true);
        setClickable(true);
        ArrayList<View> view = new ArrayList<>();
        view.add(this);
        addTouchables(view);
        setWillNotCacheDrawing(false);

        mGLThread = new GLThread(mActivity, this);
        getHolder().addCallback(mGLThread);
        mGLThread.start();
    }

    public void setClient(Client client) {
        mClient = client;
    }

    public void setServoArgs(String args, String log, boolean experimentalMode) {
        mServoArgs = args;
        mServoLog = log;
        mExperimentalMode = experimentalMode;
    }

    // RunCallback
    @Override
    public void inGLThread(Runnable r) {
        mGLLooperHandler.post(r);
    }

    @Override
    public void inUIThread(Runnable r) {
        post(r);
    }


    // GfxCallbacks
    @Override
    public void flushGLBuffers() {
    }


    @Override
    public void makeCurrent() {
    }

    // View
    @Override
    public boolean onKeyDown(int keyCode, KeyEvent event) {
        mServo.onKeyDown(keyCode, event);
        return true;
    }

    @Override
    public boolean onKeyUp(int keyCode, KeyEvent event) {
        mServo.onKeyUp(keyCode, event);
        return true;
    }

    @Override
    public boolean onTouchEvent(final MotionEvent motionEvent) {
        requestFocus();

        int action = motionEvent.getActionMasked();
        int pointerIndex = motionEvent.getActionIndex();
        int pointerId = motionEvent.getPointerId(pointerIndex);
        float x = motionEvent.getX(pointerIndex);
        float y = motionEvent.getY(pointerIndex);


        switch (action) {
            case (MotionEvent.ACTION_DOWN):
            case (MotionEvent.ACTION_POINTER_DOWN):
                mServo.touchDown(x, y, pointerId);
                break;
            case (MotionEvent.ACTION_MOVE):
                mServo.touchMove(x, y, pointerId);
                break;
            case (MotionEvent.ACTION_UP):
            case (MotionEvent.ACTION_POINTER_UP):
                mServo.touchUp(x, y, pointerId);
                break;
            case (MotionEvent.ACTION_CANCEL):
                mServo.touchCancel(x, y, pointerId);
                break;
            default:
        }

        return true;
    }

    @Override
    public void doFrame(long frameTimeNanos) {
        if (mServo != null) {
            mServo.onDoFrame();
        }
        Choreographer.getInstance().postFrameCallback(this);
    }

    // Calls from Activity
    public void onPause() {
        if (mServo != null) {
            mServo.suspend(true);
        }
    }

    public void onResume() {
        if (mServo != null) {
            mServo.suspend(false);
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

    public void mediaSessionAction(int action) {
        mServo.mediaSessionAction(action);
    }

    public void setExperimentalMode(boolean enable) {
        if (mServo != null) {
            mServo.setExperimentalMode(enable);
        }
    }

    class GLThread extends Thread implements SurfaceHolder.Callback {
        private Activity mActivity;
        private ServoView mServoView;
        GLThread(Activity activity, ServoView servoView) {
            mActivity = activity;
            mServoView = servoView;
        }

        public void surfaceCreated(SurfaceHolder holder) {
            Log.d(LOGTAG, "GLThread::surfaceCreated");

            ServoCoordinates coords = new ServoCoordinates();
            coords.width = mServoView.getWidth();
            coords.height = mServoView.getHeight();

            Surface surface = holder.getSurface();
            ServoOptions options = new ServoOptions();
            options.args = mServoView.mServoArgs;
            options.url = mServoView.mInitialUri;
            options.coordinates = coords;
            options.enableLogs = true;
            options.enableSubpixelTextAntialiasing = true;
            options.experimentalMode = mServoView.mExperimentalMode;

            DisplayMetrics metrics = new DisplayMetrics();
            mActivity.getWindowManager().getDefaultDisplay().getMetrics(metrics);
            options.density = metrics.density;
            if (mServoView.mServo == null && !mPaused) {
                mServoView.mServo = new Servo(
                        options, mServoView, mServoView, mClient, mActivity, surface);
            } else {
                mPaused = false;
                mServoView.mServo.resumePainting(surface, coords);
            }

            Choreographer.getInstance().postFrameCallback(mServoView);

        }

        public void surfaceChanged(SurfaceHolder holder, int format, int width, int height) {
            Log.d(LOGTAG, "GLThread::surfaceChanged");
            ServoCoordinates coords = new ServoCoordinates();
            coords.width = width;
            coords.height = height;
            mServoView.mServo.resize(coords);
        }

        public void surfaceDestroyed(SurfaceHolder holder) {
            Log.d(LOGTAG, "GLThread::surfaceDestroyed");
            mPaused = true;
            mServoView.mServo.pausePainting();
        }

        public void shutdown() {
            Log.d(LOGTAG, "GLThread::shutdown");
            mGLLooperHandler.getLooper().quitSafely();
        }

        public void run() {
            Looper.prepare();

            mGLLooperHandler = new Handler();

            Looper.loop();
        }
    }
}
