/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
package org.servo.servoview

import android.annotation.SuppressLint
import android.content.Context
import android.content.res.Resources
import android.util.Log
import android.util.Size
import android.view.Choreographer
import android.view.SurfaceHolder
import android.view.SurfaceView
import kotlinx.coroutines.CoroutineScope

@SuppressLint("ViewConstructor")
class ServoView(
    context: Context,
    client: Servo.Client,
    servoArgs: String?,
    servoLog: String?,
    experimentalMode: Boolean,
    initialUri: String?,
    internal val navigator: ServoNavigator,
    scope: CoroutineScope,
) : SurfaceView(context), Choreographer.FrameCallback {
    internal val servo =
        Servo(
            servoArgs,
            initialUri,
            servoLog,
            experimentalMode,
            scope,
            client,
            context,
            navigator,
        )

    init {
        isFocusable = true
        isFocusableInTouchMode = true
        addTouchables(arrayListOf(this))
        val surfaceHolderCallback = SurfaceHolderCallback(resources, servo, this)
        holder.addCallback(surfaceHolderCallback)
    }

    override fun doFrame(frameTimeNanos: Long) {
        servo.onDoFrame()
        Choreographer.getInstance().postFrameCallback(this)
    }

    fun stop() {
        servo.stop()
    }

    fun mediaSessionAction(action: Int) {
        servo.mediaSessionAction(action)
    }

    fun setExperimentalMode(enable: Boolean) {
        servo.setExperimentalMode(enable)
    }

    private class SurfaceHolderCallback(
        private val resources: Resources,
        private val servo: Servo,
        private val frameCallback: Choreographer.FrameCallback,
    ) : SurfaceHolder.Callback {
        private var paused = false

        override fun surfaceCreated(holder: SurfaceHolder) {
            Log.d(LOGTAG, "GLThread::surfaceCreated")

            val size = Size(holder.surfaceFrame.width(), holder.surfaceFrame.height())

            val surface = holder.surface

            if (!paused) {
                servo.addPlatformWindow(
                    size,
                    resources.displayMetrics.density,
                    surface,
                )
            } else {
                paused = false
                servo.resumePainting(surface, size)
            }

            Choreographer.getInstance().postFrameCallback(frameCallback)
        }

        override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
            Log.d(LOGTAG, "GLThread::surfaceChanged")
            servo.resize(Size(width, height))
        }

        override fun surfaceDestroyed(holder: SurfaceHolder) {
            Log.d(LOGTAG, "GLThread::surfaceDestroyed")
            paused = true
            servo.pausePainting()
        }
    }

    private companion object {
        private const val LOGTAG = "ServoView"
    }
}
