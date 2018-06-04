package com.mozilla.servoview;

import android.opengl.GLES31;
import android.opengl.GLSurfaceView;

import javax.microedition.khronos.egl.EGLConfig;
import javax.microedition.khronos.opengles.GL10;

public class ServoGLRenderer implements GLSurfaceView.Renderer {

    private final ServoView mView;

    ServoGLRenderer(ServoView view) {
        mView = view;
    }

    public void onSurfaceCreated(GL10 unused, EGLConfig config) {
        GLES31.glClearColor(1.0f, 0.0f, 0.0f, 1.0f);
        mView.onGLReady();
    }

    public void onDrawFrame(GL10 unused) {

    }

    public void onSurfaceChanged(GL10 unused, int width, int height) {
        GLES31.glViewport(0, 0, width, height);
        mView.onSurfaceResized(width, height);
    }
}
