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
import android.view.SurfaceHolder
import android.view.SurfaceView

@SuppressLint("ViewConstructor")
internal class ServoView(
    context: Context,
    servo: Servo,
) : SurfaceView(context) {
    init {
        val surfaceHolderCallback = SurfaceHolderCallback(resources, servo)
        holder.addCallback(surfaceHolderCallback)
    }

    private class SurfaceHolderCallback(
        private val resources: Resources,
        private val servo: Servo,
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
