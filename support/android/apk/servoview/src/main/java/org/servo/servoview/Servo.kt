/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
package org.servo.servoview

import android.content.Context
import android.util.Size
import android.view.KeyEvent
import android.view.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView

@Composable
fun Servo(
    servoView: ServoView,
    modifier: Modifier = Modifier,
) {
    AndroidView(
        factory = { _ -> servoView },
        modifier = modifier,
    )
}

class Servo(
    args: String?,
    url: String?,
    size: Size,
    density: Float,
    logStr: String?,
    enableLogs: Boolean,
    experimentalMode: Boolean,
    private val runCallback: RunCallback,
    client: Client,
    context: Context,
    surface: Surface,
) {
    private val jni = JNIServo()
    private val servoCallbacks = Callbacks(client, jni, runCallback)

    init {
        this.runCallback.inGLThread {
            jni.init(
                context,
                args,
                url,
                size,
                density,
                logStr,
                enableLogs,
                experimentalMode,
                servoCallbacks,
                surface,
            )
        }
    }

    fun version(): String {
        return jni.version()
    }

    fun performUpdates() {
        runCallback.inGLThread { jni.performUpdates() }
    }

    fun resize(size: Size) {
        runCallback.inGLThread { jni.resize(size) }
    }

    fun reload() {
        runCallback.inGLThread { jni.reload() }
    }

    fun stop() {
        runCallback.inGLThread { jni.stop() }
    }

    fun goBack() {
        runCallback.inGLThread { jni.goBack() }
    }

    fun goForward() {
        runCallback.inGLThread { jni.goForward() }
    }

    fun loadUri(uri: String) {
        runCallback.inGLThread { jni.loadUri(uri) }
    }

    fun scroll(dx: Int, dy: Int, x: Int, y: Int) {
        runCallback.inGLThread { jni.scroll(dx, dy, x, y) }
    }

    fun onKeyDown(keyCode: Int, event: KeyEvent) {
        runCallback.inGLThread { jni.keydown(keyCode, event.unicodeChar) }
    }

    fun onKeyUp(keyCode: Int, event: KeyEvent) {
        runCallback.inGLThread { jni.keyup(keyCode, event.unicodeChar) }
    }

    fun touchDown(x: Float, y: Float, pointerId: Int) {
        runCallback.inGLThread { jni.touchDown(x, y, pointerId) }
    }

    fun touchMove(x: Float, y: Float, pointerId: Int) {
        runCallback.inGLThread { jni.touchMove(x, y, pointerId) }
    }

    fun touchUp(x: Float, y: Float, pointerId: Int) {
        runCallback.inGLThread { jni.touchUp(x, y, pointerId) }
    }

    fun touchCancel(x: Float, y: Float, pointerId: Int) {
        runCallback.inGLThread { jni.touchCancel(x, y, pointerId) }
    }

    fun pinchZoomStart(factor: Float, x: Float, y: Float) {
        runCallback.inGLThread { jni.pinchZoomStart(factor, x, y) }
    }

    fun pinchZoom(factor: Float, x: Float, y: Float) {
        runCallback.inGLThread { jni.pinchZoom(factor, x, y) }
    }

    fun pinchZoomEnd(factor: Float, x: Float, y: Float) {
        runCallback.inGLThread { jni.pinchZoomEnd(factor, x, y) }
    }

    fun click(x: Float, y: Float) {
        runCallback.inGLThread { jni.click(x, y) }
    }

    fun pausePainting() {
        runCallback.inGLThread { jni.pausePainting() }
    }

    fun resumePainting(surface: Surface, size: Size) {
        runCallback.inGLThread { jni.resumePainting(surface, size) }
    }

    fun suspend(suspended: Boolean) {
        servoCallbacks.suspended = suspended
    }

    fun mediaSessionAction(action: Int) {
        runCallback.inGLThread { jni.mediaSessionAction(action) }
    }

    fun setExperimentalMode(enable: Boolean) {
        runCallback.inGLThread { jni.setExperimentalMode(enable) }
    }

    fun onDoFrame() {
        runCallback.inGLThread { jni.doFrame() }
    }

    interface Client {
        fun onAlert(message: String)

        fun onLoadStarted()

        fun onLoadEnded()

        fun onTitleChanged(title: String)

        fun onUrlChanged(url: String)

        fun onHistoryChanged(canGoBack: Boolean, canGoForward: Boolean)

        fun onImeShow()

        fun onImeHide()

        fun onMediaSessionMetadata(title: String, artist: String, album: String)

        fun onMediaSessionPlaybackStateChange(state: Int)

        fun onMediaSessionSetPositionState(duration: Float, position: Float, playbackRate: Float)
    }

    interface RunCallback {
        fun inGLThread(f: Runnable)

        fun inUIThread(f: Runnable)
    }

    private class Callbacks(
        private var client: Client,
        private val jni: JNIServo,
        private val runCallback: RunCallback,
    ) : JNIServo.Callbacks, Client {
        var suspended: Boolean = false

        override fun wakeup() {
            if (!suspended) {
                runCallback.inGLThread { jni.performUpdates() }
            }
        }

        override fun onAlert(message: String) {
            runCallback.inUIThread { client.onAlert(message) }
        }

        override fun onImeShow() {
            runCallback.inUIThread { client.onImeShow() }
        }

        override fun onImeHide() {
            runCallback.inUIThread { client.onImeHide() }
        }

        override fun onLoadStarted() {
            runCallback.inUIThread { client.onLoadStarted() }
        }

        override fun onLoadEnded() {
            runCallback.inUIThread { client.onLoadEnded() }
        }

        override fun onTitleChanged(title: String) {
            runCallback.inUIThread { client.onTitleChanged(title) }
        }

        override fun onUrlChanged(url: String) {
            runCallback.inUIThread { client.onUrlChanged(url) }
        }

        override fun onHistoryChanged(canGoBack: Boolean, canGoForward: Boolean) {
            runCallback.inUIThread { client.onHistoryChanged(canGoBack, canGoForward) }
        }

        override fun onMediaSessionMetadata(title: String, artist: String, album: String) {
            runCallback.inUIThread { client.onMediaSessionMetadata(title, artist, album) }
        }

        override fun onMediaSessionPlaybackStateChange(state: Int) {
            runCallback.inUIThread { client.onMediaSessionPlaybackStateChange(state) }
        }

        override fun onMediaSessionSetPositionState(duration: Float, position: Float, playbackRate: Float) {
            runCallback.inUIThread { client.onMediaSessionSetPositionState(duration, position, playbackRate) }
        }
    }
}
