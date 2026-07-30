/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
package org.servo.servoview

import android.content.Context
import android.util.Size
import android.view.Surface

/**
 * Maps /ports/servoshell API
 */
internal class JNIServo {
    init {
        System.loadLibrary("c++_shared")
        System.loadLibrary("servoshell")
    }

    external fun version(): String

    external fun init(
        context: Context,
        options: ServoOptions,
        callbacks: Callbacks,
        surface: Surface,
    )

    external fun performUpdates()

    external fun resize(size: Size)

    external fun reload()

    external fun stop()

    external fun goBack()

    external fun goForward()

    external fun loadUri(uri: String)

    external fun scroll(dx: Int, dy: Int, x: Int, y: Int)

    external fun keydown(keycode: Int, unicode: Int)

    external fun keyup(keycode: Int, unicode: Int)

    external fun touchDown(x: Float, y: Float, pointer_id: Int)

    external fun touchMove(x: Float, y: Float, pointer_id: Int)

    external fun touchUp(x: Float, y: Float, pointer_id: Int)

    external fun touchCancel(x: Float, y: Float, pointer_id: Int)

    external fun pinchZoomStart(factor: Float, x: Float, y: Float)

    external fun pinchZoom(factor: Float, x: Float, y: Float)

    external fun pinchZoomEnd(factor: Float, x: Float, y: Float)

    external fun click(x: Float, y: Float)

    external fun pausePainting()

    external fun resumePainting(surface: Surface, size: Size)

    external fun mediaSessionAction(action: Int)

    external fun setExperimentalMode(enable: Boolean)

    external fun doFrame()

    class ServoOptions {
        @JvmField
        var args: String? = null

        @JvmField
        var url: String? = null

        @JvmField
        var size: Size? = null

        @JvmField
        var density: Float = 1f

        @JvmField
        var logStr: String? = null

        @JvmField
        var enableLogs: Boolean = false

        @JvmField
        var experimentalMode: Boolean = false
    }

    interface Callbacks {
        fun wakeup()

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
}
