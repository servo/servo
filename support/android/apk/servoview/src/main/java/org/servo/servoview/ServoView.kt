/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
package org.servo.servoview

import android.annotation.SuppressLint
import android.content.Context
import android.util.Log
import android.util.Size
import android.view.Choreographer
import android.view.SurfaceHolder
import android.view.SurfaceView
import java.util.concurrent.Executors
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.asCoroutineDispatcher
import kotlinx.coroutines.launch

@SuppressLint("ViewConstructor")
class ServoView(
    context: Context,
    client: Servo.Client,
    servoArgs: String?,
    servoLog: String?,
    experimentalMode: Boolean,
    initialUri: String?,
    internal val navigator: ServoNavigator,
    private val scope: CoroutineScope,
) : SurfaceView(context), Servo.RunCallback, Choreographer.FrameCallback {
    internal val glDispatcher = Executors.newSingleThreadExecutor().asCoroutineDispatcher()
    internal val servo =
        Servo(
            servoArgs,
            initialUri,
            servoLog,
            experimentalMode,
            this,
            client,
            context,
            navigator,
        )

    init {
        isFocusable = true
        isFocusableInTouchMode = true
        addTouchables(arrayListOf(this))
        val surfaceHolderCallback = SurfaceHolderCallback(servoView = this)
        holder.addCallback(surfaceHolderCallback)
    }

    override fun inGLThread(r: Runnable) {
        scope.launch(glDispatcher) { r.run() }
    }

    override fun inUIThread(r: Runnable) {
        scope.launch { r.run() }
    }

    override fun doFrame(frameTimeNanos: Long) {
        servo.onDoFrame()
        Choreographer.getInstance().postFrameCallback(this)
    }

    fun stop() {
        servo.stop()
    }

    fun loadUri(uri: String) {
        servo.loadUri(uri)
    }

    fun mediaSessionAction(action: Int) {
        servo.mediaSessionAction(action)
    }

    fun setExperimentalMode(enable: Boolean) {
        servo.setExperimentalMode(enable)
    }

    private class SurfaceHolderCallback(private val servoView: ServoView) : SurfaceHolder.Callback {
        private var paused = false

        override fun surfaceCreated(holder: SurfaceHolder) {
            Log.d(LOGTAG, "GLThread::surfaceCreated")

            val size = Size(servoView.width, servoView.height)

            val surface = holder.surface

            if (!paused) {
                servoView.servo.addPlatformWindow(
                    size,
                    servoView.resources.displayMetrics.density,
                    servoView,
                    surface,
                )
            } else {
                paused = false
                servoView.servo.resumePainting(surface, size)
            }

            Choreographer.getInstance().postFrameCallback(servoView)
        }

        override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
            Log.d(LOGTAG, "GLThread::surfaceChanged")
            servoView.servo.resize(Size(width, height))
        }

        override fun surfaceDestroyed(holder: SurfaceHolder) {
            Log.d(LOGTAG, "GLThread::surfaceDestroyed")
            paused = true
            servoView.servo.pausePainting()
        }
    }

    private companion object {
        private const val LOGTAG = "ServoView"
    }
}
