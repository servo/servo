/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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
        mView.onGLReady();
    }

    public void onDrawFrame(GL10 unused) {
    }

    public void onSurfaceChanged(GL10 unused, int width, int height) {
        GLES31.glViewport(0, 0, width, height);
        mView.onSurfaceResized(width, height);
    }
}
