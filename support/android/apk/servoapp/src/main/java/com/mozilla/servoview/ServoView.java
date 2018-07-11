/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.servoview;

import android.app.Activity;
import android.os.Build;
import android.content.Context;
import android.content.res.AssetManager;
import android.net.Uri;
import android.opengl.GLSurfaceView;
import android.util.AttributeSet;
import android.util.Log;
import android.view.Choreographer;
import android.view.GestureDetector;
import android.view.MotionEvent;
import android.widget.OverScroller;
import java.io.IOException;
import java.io.InputStream;

public class ServoView extends GLSurfaceView implements GestureDetector.OnGestureListener, Choreographer.FrameCallback {

    private static final String LOGTAG = "ServoView";

    private Activity mActivity;
    private NativeServo mServo;
    private Client mClient = null;
    private Uri mInitialUri = null;
    private boolean mAnimating;
    private String mServoArgs = "";

    public ServoView(Context context, AttributeSet attrs) {
        super(context, attrs);
        mActivity = (Activity) context;
        setFocusable(true);
        setFocusableInTouchMode(true);
        setWillNotCacheDrawing(false);
        setEGLContextClientVersion(3);
        setEGLConfigChooser(8, 8, 8, 8, 24, 0);
        ServoGLRenderer mRenderer = new ServoGLRenderer(this);
        setRenderer(mRenderer);
        mServo = new NativeServo();
        setRenderMode(GLSurfaceView.RENDERMODE_WHEN_DIRTY);
        initGestures(context);
    }

    public void setServoArgs(String args) {
      mServoArgs = args;
    }

    public void reload() {
        queueEvent(() -> mServo.reload());
    }

    public void goBack() {
        queueEvent(() -> mServo.goBack());
    }

    public void goForward() {
        queueEvent(() -> mServo.goForward());
    }

    public void stop() {
        queueEvent(() -> mServo.stop());
    }

    public void onSurfaceResized(int width, int height) {
        queueEvent(() -> mServo.resize(width, height));
    }

    public void loadUri(Uri uri) {
        if (mServo != null) {
            queueEvent(() -> mServo.loadUri(uri.toString()));
        } else {
            mInitialUri = uri;
        }
    }

    class WakeupCallback implements NativeServo.WakeupCallback {
        public void wakeup() {
            queueEvent(() -> mServo.performUpdates());
        };
    }

    class ReadFileCallback implements NativeServo.ReadFileCallback {
        public byte[] readfile(String file) {
            try {
                AssetManager assetMgr = getContext().getResources().getAssets();
                InputStream stream = assetMgr.open(file);
                byte[] bytes = new byte[stream.available()];
                stream.read(bytes);
                stream.close();
                return bytes;
            } catch (IOException e) {
                Log.e(LOGTAG, e.getMessage());
                return null;
            }
        }
    }

    class ServoCallbacks implements NativeServo.ServoCallbacks {
        public void flush() {
            requestRender();
        }

        public void onLoadStarted() {
            if (mClient != null) {
                post(() -> mClient.onLoadStarted());
            }
        }

        public void onLoadEnded() {
            if (mClient != null) {
                post(() -> mClient.onLoadEnded());
            }
        }

        public void onTitleChanged(final String title) {
            if (mClient != null) {
                post(() -> mClient.onTitleChanged(title));
            }
        }

        public void onUrlChanged(final String url) {
            if (mClient != null) {
                post(() -> mClient.onUrlChanged(url));
            }
        }

        public void onHistoryChanged(final boolean canGoBack, final boolean canGoForward) {
            if (mClient != null) {
                post(() -> mClient.onHistoryChanged(canGoBack, canGoForward));
            }
        }

        public void onAnimatingChanged(final boolean animating) {
            if (!mAnimating && animating) {
                post(() -> Choreographer.getInstance().postFrameCallback(ServoView.this));
            }
            mAnimating = animating;
        }
    }

    public void onGLReady() {
        final WakeupCallback c1 = new WakeupCallback();
        final ReadFileCallback c2 = new ReadFileCallback();
        final ServoCallbacks c3 = new ServoCallbacks();
        final boolean showLogs = true;
        int width = getWidth();
        int height = getHeight();
        queueEvent(() -> {
            String uri = mInitialUri == null ? null : mInitialUri.toString();
            mServo.init(mActivity, mServoArgs, uri, c1, c2, c3, width, height, showLogs);
        });
    }

    public interface Client {
        void onLoadStarted();
        void onLoadEnded();
        void onTitleChanged(String title);
        void onUrlChanged(String url);
        void onHistoryChanged(boolean canGoBack, boolean canGoForward);
    }

    public void setClient(Client client) {
        mClient = client;
    }

    // Scroll and click

    private GestureDetector mGestureDetector;
    private OverScroller mScroller;
    private int mLastX = 0;
    private int mCurX = 0;
    private int mLastY = 0;
    private int mCurY = 0;
    private boolean mFlinging;

    private void initGestures(Context context) {
        mGestureDetector = new GestureDetector(context, this);
        mScroller = new OverScroller(context);
    }

    @Override
    public void doFrame(long frameTimeNanos) {

        if (mScroller.isFinished() && mFlinging) {
            mFlinging = false;
            queueEvent(() -> mServo.scrollEnd(0, 0, mCurX, mCurY));
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
            queueEvent(() -> mServo.scroll(dx, dy, mCurX, mCurY));
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
        mScroller.fling(mCurX, mCurY, (int)velocityX, (int)velocityY, 0, mPageWidth, 0, mPageHeight);
        return true;
    }

    public boolean onDown(MotionEvent e) {
        mScroller.forceFinished(true);
        return true;
    }

    public boolean onTouchEvent(final MotionEvent e) {
        mGestureDetector.onTouchEvent(e);

        int action = e.getActionMasked();
        switch(action) {
            case (MotionEvent.ACTION_DOWN):
                mCurX = (int)e.getX();
                mLastX = mCurX;
                mCurY = (int)e.getY();
                mLastY = mCurY;
                mScroller.forceFinished(true);
                queueEvent(() -> mServo.scrollStart(0, 0, mCurX, mCurY));
                Choreographer.getInstance().postFrameCallback(this);
                return true;
            case (MotionEvent.ACTION_MOVE):
                mCurX = (int)e.getX();
                mCurY = (int)e.getY();
                return true;
            case (MotionEvent.ACTION_UP):
            case (MotionEvent.ACTION_CANCEL):
                if (!mFlinging) {
                    queueEvent(() -> mServo.scrollEnd(0, 0, mCurX, mCurY));
                    Choreographer.getInstance().removeFrameCallback(this);
                }
                return true;
            default:
                return true;
        }
    }

    public boolean onSingleTapUp(MotionEvent e) {
        queueEvent(() -> mServo.click((int)e.getX(), (int)e.getY()));
        return false;
    }

    public void onLongPress(MotionEvent e) { }
    public boolean onScroll(MotionEvent e1, MotionEvent e2, float distanceX, float distanceY) { return true; }
    public void onShowPress(MotionEvent e) { }

}
