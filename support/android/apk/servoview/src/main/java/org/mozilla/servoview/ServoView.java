/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

package org.mozilla.servoview;

import android.app.Activity;
import android.content.Context;
import android.net.Uri;
import android.opengl.GLES31;
import android.opengl.GLSurfaceView;
import android.util.AttributeSet;
import android.util.DisplayMetrics;
import android.util.Log;
import android.view.Choreographer;
import android.view.GestureDetector;
import android.view.MotionEvent;
import android.view.ScaleGestureDetector;
import android.widget.OverScroller;

import org.mozilla.servoview.JNIServo.ServoCoordinates;
import org.mozilla.servoview.JNIServo.ServoOptions;
import org.mozilla.servoview.Servo.Client;
import org.mozilla.servoview.Servo.GfxCallbacks;
import org.mozilla.servoview.Servo.RunCallback;

import javax.microedition.khronos.egl.EGLConfig;
import javax.microedition.khronos.opengles.GL10;

public class ServoView extends GLSurfaceView
        implements
        GestureDetector.OnGestureListener,
        ScaleGestureDetector.OnScaleGestureListener,
        Choreographer.FrameCallback,
        GfxCallbacks,
        RunCallback {

    private static final String LOGTAG = "ServoView";

    private Activity mActivity;
    private Servo mServo;
    private Client mClient = null;
    private Uri mInitialUri = null;
    private boolean mAnimating;
    private String mServoArgs;
    private String mServoLog;
    private String mGstDebug;
    private GestureDetector mGestureDetector;
    private ScaleGestureDetector mScaleGestureDetector;

    private OverScroller mScroller;
    private int mLastX = 0;
    private int mCurX = 0;
    private int mLastY = 0;
    private int mCurY = 0;
    private boolean mFlinging;

    private boolean mZooming;
    private float mZoomFactor = 1;

    private boolean mRedrawing;

    public ServoView(Context context) {
        super(context);
        init(context);
    }

    public ServoView(Context context, AttributeSet attrs) {
        super(context, attrs);
        init(context);
    }

    public void onDetachedFromWindow() {
        mServo.shutdown();
        mServo = null;
        super.onDetachedFromWindow();
    }

    private void init(Context context) {
        mActivity = (Activity) context;
        setFocusable(true);
        setFocusableInTouchMode(true);
        setWillNotCacheDrawing(false);
        setEGLContextClientVersion(3);
        setEGLConfigChooser(8, 8, 8, 8, 24, 0);
        setPreserveEGLContextOnPause(true);
        ServoGLRenderer mRenderer = new ServoGLRenderer(this);
        setRenderer(mRenderer);
        setRenderMode(GLSurfaceView.RENDERMODE_WHEN_DIRTY);
        initGestures(context);
    }

    public void setServoArgs(String args, String log, String gstdebug) {
        mServoArgs = args;
        mServoLog = log;
        mGstDebug = gstdebug;
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

    public void onSurfaceInvalidated(int width, int height) {
        if (mServo != null) {
            ServoCoordinates coords = new ServoCoordinates();
            coords.width = width;
            coords.height = height;
            coords.fb_width = width;
            coords.fb_height = height;

            mServo.resize(coords);
            mServo.refresh();
        }
    }

    public void loadUri(Uri uri) {
        if (mServo != null) {
            mServo.loadUri(uri.toString());
        } else {
            mInitialUri = uri;
        }
    }

    public void mediaSessionAction(int action) {
        mServo.mediaSessionAction(action);
    }

    public void flushGLBuffers() {
        requestRender();
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

    public void inGLThread(Runnable f) {
        queueEvent(f);
    }

    public void inUIThread(Runnable f) {
        post(f);
    }

    public void onGLReady() {
        ServoCoordinates coords = new ServoCoordinates();
        coords.width = getWidth();
        coords.height = getHeight();
        coords.fb_width = getWidth();
        coords.fb_height = getHeight();

        ServoOptions options = new ServoOptions();
        options.args = mServoArgs;
        options.coordinates = coords;
        options.enableLogs = true;
        options.enableSubpixelTextAntialiasing = true;

        DisplayMetrics metrics = new DisplayMetrics();
        mActivity.getWindowManager().getDefaultDisplay().getMetrics(metrics);
        options.density = metrics.density;
        inGLThread(() -> {
            String uri = mInitialUri == null ? null : mInitialUri.toString();
            options.url = uri;
            options.logStr = mServoLog;
            options.gstDebugStr = mGstDebug;
            mServo = new Servo(options, this, this, mClient, mActivity);
        });
    }

    public void setClient(Client client) {
        mClient = client;
    }

    private void initGestures(Context context) {
        mGestureDetector = new GestureDetector(context, this);
        mScaleGestureDetector = new ScaleGestureDetector(context, this);
        mScroller = new OverScroller(context);
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
                mServo.touchDown(x, y, pointerId);
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
                mServo.touchMove(x, y, pointerId);
                return true;
            case (MotionEvent.ACTION_UP):
                mServo.touchUp(x, y, pointerId);
            case (MotionEvent.ACTION_CANCEL):
                mServo.touchCancel(x, y, pointerId);
                return true;
            default:
                return true;
        }
    }

    public boolean onSingleTapUp(MotionEvent e) {
        return false;
    }

    public void onLongPress(MotionEvent e) {
    }

    public boolean onScroll(MotionEvent e1, MotionEvent e2, float distanceX, float distanceY) {
        return true;
    }

    public void onShowPress(MotionEvent e) {
    }

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


    @Override
    public void onPause() {
      super.onPause();
      if (mServo != null) {
        mServo.suspend(true);
      }
    }

    @Override
    public void onResume() {
      super.onResume();
      if (mServo != null) {
        mServo.suspend(false);
      }
    }

    static class ServoGLRenderer implements Renderer {

        private final ServoView mView;

        ServoGLRenderer(ServoView view) {
            mView = view;
        }

        public void onSurfaceCreated(GL10 unused, EGLConfig config) {
            mView.onGLReady();
        }

        public void onDrawFrame(GL10 unused) {
        }

        public void onSurfaceChanged(GL10 gl, int width, int height) {
            GLES31.glViewport(0, 0, width, height);
            mView.onSurfaceInvalidated(width, height);
        }
    }
}
