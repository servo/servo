/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
package org.servo.servoview

import android.annotation.SuppressLint
import android.content.Context
import android.os.Handler
import android.os.Looper
import android.util.Log
import android.util.Size
import android.view.Choreographer
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.SurfaceHolder
import android.view.SurfaceView

@SuppressLint("ViewConstructor")
class ServoView(
    context: Context,
    client: Servo.Client,
) : SurfaceView(context), Servo.RunCallback, Choreographer.FrameCallback {
    private val glThread: GLThread
    private val surfaceHolderCallback: SurfaceHolderCallback
    private var servo: Servo? = null
    private var servoArgs: String? = null
    private var initialUri: String? = null

    private var experimentalMode = false

    init {
        isFocusable = true
        isFocusableInTouchMode = true
        isClickable = true
        addTouchables(arrayListOf(this))
        glThread = GLThread()
        surfaceHolderCallback = SurfaceHolderCallback(this, client)
        holder.addCallback(surfaceHolderCallback)
        glThread.start()
    }

    fun setServoArgs(args: String?, log: String?, experimentalMode: Boolean) {
        servoArgs = args
        surfaceHolderCallback.servoLog = log
        this.experimentalMode = experimentalMode
    }

    override fun inGLThread(r: Runnable) {
        glThread.glLooperHandler!!.post(r)
    }

    override fun inUIThread(r: Runnable) {
        post(r)
    }

    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean {
        if (event.keyCode != KeyEvent.KEYCODE_BACK) {
            servo!!.onKeyDown(keyCode, event)
            return true
        }
        return false
    }

    override fun onKeyUp(keyCode: Int, event: KeyEvent): Boolean {
        if (event.keyCode != KeyEvent.KEYCODE_BACK) {
            servo!!.onKeyUp(keyCode, event)
            return true
        }
        return false
    }

    override fun onTouchEvent(motionEvent: MotionEvent): Boolean {
        requestFocus()

        val action = motionEvent.actionMasked
        val pointerIndex = motionEvent.actionIndex
        val pointerId = motionEvent.getPointerId(pointerIndex)
        val x = motionEvent.getX(pointerIndex)
        val y = motionEvent.getY(pointerIndex)

        when (action) {
            MotionEvent.ACTION_DOWN, MotionEvent.ACTION_POINTER_DOWN -> servo!!.touchDown(x, y, pointerId)
            MotionEvent.ACTION_MOVE -> servo!!.touchMove(x, y, pointerId)
            MotionEvent.ACTION_UP, MotionEvent.ACTION_POINTER_UP -> servo!!.touchUp(x, y, pointerId)
            MotionEvent.ACTION_CANCEL -> servo!!.touchCancel(x, y, pointerId)
        }

        return true
    }

    override fun doFrame(frameTimeNanos: Long) {
        servo?.onDoFrame()
        Choreographer.getInstance().postFrameCallback(this)
    }

    fun onPause() {
        servo?.suspend(true)
    }

    fun onResume() {
        servo?.suspend(false)
    }

    fun reload() {
        servo!!.reload()
    }

    fun goBack() {
        servo!!.goBack()
    }

    fun goForward() {
        servo!!.goForward()
    }

    fun stop() {
        servo!!.stop()
    }

    fun loadUri(uri: String) {
        val servo = servo
        if (servo != null) {
            servo.loadUri(uri)
        } else {
            initialUri = uri
        }
    }

    fun mediaSessionAction(action: Int) {
        servo!!.mediaSessionAction(action)
    }

    fun setExperimentalMode(enable: Boolean) {
        servo?.setExperimentalMode(enable)
    }

    private class GLThread : Thread() {
        var glLooperHandler: Handler? = null

        override fun run() {
            Looper.prepare()

            glLooperHandler = Handler(Looper.myLooper()!!)

            Looper.loop()
        }
    }

    private class SurfaceHolderCallback(
        private val servoView: ServoView,
        private val client: Servo.Client,
    ) : SurfaceHolder.Callback {
        var servoLog: String? = null
        private var paused = false

        override fun surfaceCreated(holder: SurfaceHolder) {
            Log.d(LOGTAG, "GLThread::surfaceCreated")

            val size = Size(servoView.width, servoView.height)

            val surface = holder.surface

            if (servoView.servo == null && !paused) {
                servoView.servo = Servo(
                    servoView.servoArgs,
                    servoView.initialUri,
                    size,
                    servoView.resources.displayMetrics.density,
                    servoLog,
                    true,
                    servoView.experimentalMode,
                    servoView,
                    client,
                    servoView.context,
                    surface,
                )
            } else {
                paused = false
                servoView.servo!!.resumePainting(surface, size)
            }

            Choreographer.getInstance().postFrameCallback(servoView)
        }

        override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
            Log.d(LOGTAG, "GLThread::surfaceChanged")
            servoView.servo!!.resize(Size(width, height))
        }

        override fun surfaceDestroyed(holder: SurfaceHolder) {
            Log.d(LOGTAG, "GLThread::surfaceDestroyed")
            paused = true
            servoView.servo!!.pausePainting()
        }
    }

    private companion object {
        private const val LOGTAG = "ServoView"
    }
}
