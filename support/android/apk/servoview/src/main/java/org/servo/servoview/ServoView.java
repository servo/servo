/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

package org.servo.servoview;

import android.content.Context;
import android.os.Handler;
import android.os.Looper;
import android.util.AttributeSet;
import android.util.DisplayMetrics;
import android.util.Log;
import android.view.Choreographer;
import android.view.KeyEvent;
import android.view.MotionEvent;
import android.view.Surface;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import android.view.View;

import org.servo.servoview.JNIServo.ServoCoordinates;
import org.servo.servoview.JNIServo.ServoOptions;
import org.servo.servoview.Servo.Client;
import org.servo.servoview.Servo.RunCallback;

import java.util.ArrayList;

public class ServoView extends SurfaceView
        implements
        RunCallback,
        Choreographer.FrameCallback {
    private static final String LOGTAG = "ServoView";
    private GLThread glThread;
    private Handler glLooperHandler;
    private Surface aSurface;
    protected Servo servo = null;
    private Client client = null;
    private String servoArgs;
    private String servoLog;
    private String initialUri;

    private boolean experimentalMode;
    private boolean paused = false;

    public ServoView(Context context) {
        super(context);
        init();
    }

    public ServoView(Context context, AttributeSet attrs) {
        super(context, attrs);
        init();
    }

    private void init() {
        setFocusable(true);
        setFocusableInTouchMode(true);
        setClickable(true);
        ArrayList<View> view = new ArrayList<>();
        view.add(this);
        addTouchables(view);

        glThread = new GLThread(this);
        getHolder().addCallback(glThread);
        glThread.start();
    }

    public void setClient(Client client) {
        this.client = client;
    }

    public void setServoArgs(String args, String log, boolean experimentalMode) {
        servoArgs = args;
        servoLog = log;
        this.experimentalMode = experimentalMode;
    }

    // RunCallback
    @Override
    public void inGLThread(Runnable r) {
        glLooperHandler.post(r);
    }

    @Override
    public void inUIThread(Runnable r) {
        post(r);
    }

    // View
    @Override
    public boolean onKeyDown(int keyCode, KeyEvent event) {
        if (event.getKeyCode() != KeyEvent.KEYCODE_BACK) {
            servo.onKeyDown(keyCode, event);
            return true;
        }
        return false;
    }

    @Override
    public boolean onKeyUp(int keyCode, KeyEvent event) {
        if (event.getKeyCode() != KeyEvent.KEYCODE_BACK) {
            servo.onKeyUp(keyCode, event);
            return true;
        }
        return false;
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
                servo.touchDown(x, y, pointerId);
                break;
            case (MotionEvent.ACTION_MOVE):
                servo.touchMove(x, y, pointerId);
                break;
            case (MotionEvent.ACTION_UP):
            case (MotionEvent.ACTION_POINTER_UP):
                servo.touchUp(x, y, pointerId);
                break;
            case (MotionEvent.ACTION_CANCEL):
                servo.touchCancel(x, y, pointerId);
                break;
            default:
        }

        return true;
    }

    @Override
    public void doFrame(long frameTimeNanos) {
        if (servo != null) {
            servo.onDoFrame();
        }
        Choreographer.getInstance().postFrameCallback(this);
    }

    // Calls from Activity
    public void onPause() {
        if (servo != null) {
            servo.suspend(true);
        }
    }

    public void onResume() {
        if (servo != null) {
            servo.suspend(false);
        }
    }

    public void reload() {
        servo.reload();
    }

    public void goBack() {
        servo.goBack();
    }

    public void goForward() {
        servo.goForward();
    }

    public void stop() {
        servo.stop();
    }

    public void loadUri(String uri) {
        if (servo != null) {
            servo.loadUri(uri);
        } else {
            initialUri = uri;
        }
    }

    public void mediaSessionAction(int action) {
        servo.mediaSessionAction(action);
    }

    public void setExperimentalMode(boolean enable) {
        if (servo != null) {
            servo.setExperimentalMode(enable);
        }
    }

    class GLThread extends Thread implements SurfaceHolder.Callback {
        private ServoView servoView;

        GLThread(ServoView servoView) {
            this.servoView = servoView;
        }

        public void surfaceCreated(SurfaceHolder holder) {
            Log.d(LOGTAG, "GLThread::surfaceCreated");

            ServoCoordinates coords = new ServoCoordinates();
            coords.width = servoView.getWidth();
            coords.height = servoView.getHeight();

            Surface surface = holder.getSurface();
            ServoOptions options = new ServoOptions();
            options.args = servoView.servoArgs;
            options.url = servoView.initialUri;
            options.coordinates = coords;
            options.logStr = servoLog;
            options.enableLogs = true;
            options.experimentalMode = servoView.experimentalMode;

            DisplayMetrics metrics = servoView.getResources().getDisplayMetrics();
            options.density = metrics.density;
            if (servoView.servo == null && !paused) {
                servoView.servo = new Servo(
                        options, servoView, client, servoView.getContext(), surface);
            } else {
                paused = false;
                servoView.servo.resumePainting(surface, coords);
            }

            Choreographer.getInstance().postFrameCallback(servoView);

        }

        public void surfaceChanged(SurfaceHolder holder, int format, int width, int height) {
            Log.d(LOGTAG, "GLThread::surfaceChanged");
            ServoCoordinates coords = new ServoCoordinates();
            coords.width = width;
            coords.height = height;
            servoView.servo.resize(coords);
        }

        public void surfaceDestroyed(SurfaceHolder holder) {
            Log.d(LOGTAG, "GLThread::surfaceDestroyed");
            paused = true;
            servoView.servo.pausePainting();
        }

        public void shutdown() {
            Log.d(LOGTAG, "GLThread::shutdown");
            glLooperHandler.getLooper().quitSafely();
        }

        public void run() {
            Looper.prepare();

            glLooperHandler = new Handler(Looper.myLooper());

            Looper.loop();
        }
    }
}
