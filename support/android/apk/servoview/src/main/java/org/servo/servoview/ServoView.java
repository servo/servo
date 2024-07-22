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
                        Choreographer.FrameCallback,
                        GestureDetector.OnGestureListener,
                        ScaleGestureDetector.OnScaleGestureListener {
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
    private GestureDetector mGestureDetector;
    private int mLastX = 0;
    private int mCurX = 0;
    private int mLastY = 0;
    private int mCurY = 0;
    private boolean mFlinging;
    private ScaleGestureDetector mScaleGestureDetector;
    private OverScroller mScroller;

    private boolean mZooming;
    private float mZoomFactor = 1;
    private boolean mRedrawing;
    private boolean mAnimating;
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
        initGestures(context);

        mGLThread = new GLThread(mActivity, this);
        getHolder().addCallback(mGLThread);
        mGLThread.start();
    }

    public void setClient(Client client) {
        mClient = client;
    }

    public void setServoArgs(String args, String log) {
        mServoArgs = args;
        mServoLog = log;
    }

    // RunCallback
    public void inGLThread(Runnable r) {
        mGLLooperHandler.post(r);
    }

    public void inUIThread(Runnable r) {
        post(r);
    }


    // GfxCallbacks
    public void flushGLBuffers() {
    }


    // Scroll and click
    public void animationStateChanged(boolean animating) {
        if (!mAnimating && animating) {
            post(() -> startLooping());
        }
        mAnimating = animating;
    }

    public void makeCurrent() {
    }


    private void startLooping() {
      // In case we were already drawing.
      Choreographer.getInstance().removeFrameCallback(this);

      Choreographer.getInstance().postFrameCallback(this);
    }

    public void doFrame(long frameTimeNanos) {
        if (!mRedrawing) {
            mRedrawing = true;
            mClient.onRedrawing(mRedrawing);
        }

        // 3 reasons to be here: animating or scrolling/flinging or pinching

        if (mFlinging && mScroller.isFinished()) {
            mFlinging = false;
            mServo.scrollEnd(0, 0, mCurX, mCurY);
        }

        if (mFlinging) {
            mScroller.computeScrollOffset();
            mCurX = mScroller.getCurrX();
            mCurY = mScroller.getCurrY();
        }

        int dx = mCurX - mLastX;
        int dy = mCurY - mLastY;

        mLastX = mCurX;
        mLastY = mCurY;

        boolean scrollNecessary = mFlinging && (dx != 0 || dy != 0);
        boolean zoomNecessary = mZooming && mZoomFactor != 1;

        if (scrollNecessary) {
            mServo.scroll(dx, dy, mCurX, mCurY);
        }

        if (zoomNecessary) {
            mServo.pinchZoom(mZoomFactor, 0, 0);
            mZoomFactor = 1;
        }

        if (!zoomNecessary && !scrollNecessary && mAnimating) {
            mServo.performUpdates();
        }

        if (mZooming || mFlinging || mAnimating) {
            Choreographer.getInstance().postFrameCallback(this);
        } else {
            mRedrawing = false;
            mClient.onRedrawing(mRedrawing);
        }
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

    public boolean onFling(MotionEvent e1, MotionEvent e2, float velocityX, float velocityY) {
        mFlinging = true;

        // FIXME: magic values
        // https://github.com/servo/servo/issues/20361
        int mPageWidth = 80000;
        int mPageHeight = 80000;
        mCurX = velocityX < 0 ? mPageWidth : 0;
        mLastX = mCurX;
        mCurY = velocityY < 0 ? mPageHeight : 0;
        mLastY = mCurY;
        mScroller.fling(mCurX, mCurY, (int) velocityX, (int) velocityY, 0, mPageWidth, 0, mPageHeight);
        mServo.scrollStart(0, 0, mCurX, mCurY);
        startLooping();
        return true;
    }

    public boolean onDown(MotionEvent e) {
        mScroller.forceFinished(true);
        return true;
    }

    @Override
    public boolean onTouchEvent(final MotionEvent e) {
        mGestureDetector.onTouchEvent(e);
        mScaleGestureDetector.onTouchEvent(e);

        int action = e.getActionMasked();

        float x = e.getX();
        float y = e.getY();

        int pointerIndex = e.getActionIndex();
        int pointerId = e.getPointerId(pointerIndex);

        switch (action) {
            case (MotionEvent.ACTION_DOWN):
            case (MotionEvent.ACTION_POINTER_DOWN):
                mFlinging = false;
                mScroller.forceFinished(true);
                mCurX = (int) x;
                mLastX = mCurX;
                mCurY = (int) y;
                mLastY = mCurY;
                return true;
            case (MotionEvent.ACTION_MOVE):
                mCurX = (int) x;
                mCurY = (int) y;
                return true;
            case (MotionEvent.ACTION_UP):
            case (MotionEvent.ACTION_POINTER_UP):
                return true;
            case (MotionEvent.ACTION_CANCEL):
                return true;
            default:
                return true;
        }
    }

    // OnGestureListener
    public void onLongPress(MotionEvent e) {
    }

    public boolean onScroll(MotionEvent e1, MotionEvent e2, float distanceX, float distanceY) {
        mServo.scroll((int) -distanceX, (int) -distanceY, (int) e1.getX(), (int) e1.getY());
        return true;
    }

    public boolean onSingleTapUp(MotionEvent e) {
        click(e.getX(), e.getY());
        return false;
    }

    public void onShowPress(MotionEvent e) {
    }

    // OnScaleGestureListener
    @Override
    public boolean onScaleBegin(ScaleGestureDetector detector) {
        if (mScroller.isFinished()) {
            mZoomFactor = detector.getScaleFactor();
            mZooming = true;
            mServo.pinchZoomStart(mZoomFactor, 0, 0);
            startLooping();
            return true;
        } else {
            return false;
        }
    }

    @Override
    public boolean onScale(ScaleGestureDetector detector) {
        mZoomFactor *= detector.getScaleFactor();
        return true;
    }

    @Override
    public void onScaleEnd(ScaleGestureDetector detector) {
        mZoomFactor = detector.getScaleFactor();
        mZooming = false;
        mServo.pinchZoomEnd(mZoomFactor, 0, 0);
    }

    private void initGestures(Context context) {
        mGestureDetector = new GestureDetector(context, this);
        mScaleGestureDetector = new ScaleGestureDetector(context, this);
        mScroller = new OverScroller(context);
    }

    public void mediaSessionAction(int action) {
        mServo.mediaSessionAction(action);
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
            coords.fb_width = mServoView.getWidth();
            coords.fb_height = mServoView.getHeight();

            Surface surface = holder.getSurface();
            ServoOptions options = new ServoOptions();
            options.args = mServoView.mServoArgs;
            options.url = mServoView.mInitialUri;
            options.coordinates = coords;
            options.enableLogs = true;
            options.enableSubpixelTextAntialiasing = true;

            DisplayMetrics metrics = new DisplayMetrics();
            mActivity.getWindowManager().getDefaultDisplay().getMetrics(metrics);
            options.density = metrics.density;
            if (mServoView.mServo == null && !mPaused) {
                mServoView.mServo = new Servo(
                        options, mServoView, mServoView, mClient, mActivity, surface);
            } else {
                mPaused = false;
                mServoView.mServo.resumeCompositor(surface, coords);
            }

        }

        public void surfaceChanged(SurfaceHolder holder, int format, int width, int height) {
            Log.d(LOGTAG, "GLThread::surfaceChanged");
            ServoCoordinates coords = new ServoCoordinates();
            coords.width = width;
            coords.height = height;
            coords.fb_width = width;
            coords.fb_height = height;

            mServoView.mServo.resize(coords);
        }

        public void surfaceDestroyed(SurfaceHolder holder) {
            Log.d(LOGTAG, "GLThread::surfaceDestroyed");
            mPaused = true;
            mServoView.mServo.pauseCompositor();
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
