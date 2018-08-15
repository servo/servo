/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.servoview;

import android.app.Activity;
import android.content.Context;
import android.net.Uri;
import android.opengl.GLES31;
import android.opengl.GLSurfaceView;
import android.util.AttributeSet;
import android.view.Choreographer;
import android.view.GestureDetector;
import android.view.MotionEvent;
import android.widget.OverScroller;

import com.mozilla.servoview.Servo.Client;
import com.mozilla.servoview.Servo.GfxCallbacks;
import com.mozilla.servoview.Servo.RunCallback;

import javax.microedition.khronos.egl.EGLConfig;
import javax.microedition.khronos.opengles.GL10;

public class ServoView extends GLSurfaceView
        implements
        GestureDetector.OnGestureListener,
        Choreographer.FrameCallback,
        GfxCallbacks,
        RunCallback {

    private static final String LOGTAG = "ServoView";

    private Activity mActivity;
    private Servo mServo;
    private Client mClient = null;
    private Uri mInitialUri = null;
    private boolean mAnimating;
    private String mServoArgs = "";
    private GestureDetector mGestureDetector;
    private OverScroller mScroller;
    private int mLastX = 0;
    private int mCurX = 0;
    private int mLastY = 0;
    private int mCurY = 0;
    private boolean mFlinging;

    public ServoView(Context context, AttributeSet attrs) {
        super(context, attrs);
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

    public void setServoArgs(String args) {
        mServoArgs = args != null ? args : "";
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
            mServo.resize(width, height);
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

    public void flushGLBuffers() {
      inUIThread(() -> {
        requestRender();
      });
    }

    // Scroll and click

    public void animationStateChanged(boolean animating) {
        if (!mAnimating && animating) {
            post(() -> Choreographer.getInstance().postFrameCallback(ServoView.this));
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
        final boolean showLogs = true;
        int width = getWidth();
        int height = getHeight();
        inGLThread(() -> {
            String uri = mInitialUri == null ? null : mInitialUri.toString();
            mServo = new Servo(this, this, mClient, mActivity, mServoArgs, uri, width, height, showLogs);
        });
    }

    public void setClient(Client client) {
        mClient = client;
    }

    private void initGestures(Context context) {
        mGestureDetector = new GestureDetector(context, this);
        mScroller = new OverScroller(context);
    }

    public void doFrame(long frameTimeNanos) {

        if (mScroller.isFinished() && mFlinging) {
            mFlinging = false;
            inGLThread(() -> mServo.scrollEnd(0, 0, mCurX, mCurY));
            if (!mAnimating) {
                // Not scrolling. Not animating. We don't need to schedule
                // another frame.
                return;
            }
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

        if (dx != 0 || dy != 0) {
            inGLThread(() -> mServo.scroll(dx, dy, mCurX, mCurY));
        } else {
            if (mAnimating) {
                requestRender();
            }
        }

        Choreographer.getInstance().postFrameCallback(this);
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
        return true;
    }

    public boolean onDown(MotionEvent e) {
        mScroller.forceFinished(true);
        return true;
    }

    public boolean onTouchEvent(final MotionEvent e) {
        mGestureDetector.onTouchEvent(e);

        int action = e.getActionMasked();
        switch (action) {
            case (MotionEvent.ACTION_DOWN):
                mCurX = (int) e.getX();
                mLastX = mCurX;
                mCurY = (int) e.getY();
                mLastY = mCurY;
                mScroller.forceFinished(true);
                inGLThread(() -> mServo.scrollStart(0, 0, mCurX, mCurY));
                Choreographer.getInstance().postFrameCallback(this);
                return true;
            case (MotionEvent.ACTION_MOVE):
                mCurX = (int) e.getX();
                mCurY = (int) e.getY();
                return true;
            case (MotionEvent.ACTION_UP):
            case (MotionEvent.ACTION_CANCEL):
                if (!mFlinging) {
                    inGLThread(() -> mServo.scrollEnd(0, 0, mCurX, mCurY));
                    Choreographer.getInstance().removeFrameCallback(this);
                }
                return true;
            default:
                return true;
        }
    }

    public boolean onSingleTapUp(MotionEvent e) {
        inGLThread(() -> mServo.click((int) e.getX(), (int) e.getY()));
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
