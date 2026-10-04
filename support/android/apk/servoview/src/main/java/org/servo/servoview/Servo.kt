/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
package org.servo.servoview

import android.content.Context
import android.util.Size
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.Surface
import androidx.compose.foundation.interaction.Interaction
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.nativeKeyCode
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.input.pointer.pointerInteropFilter
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.compose.LifecycleResumeEffect
import java.util.concurrent.Executors
import kotlin.coroutines.EmptyCoroutineContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.asCoroutineDispatcher
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.launch

@Composable
fun Servo(
    servoView: ServoView,
    modifier: Modifier = Modifier,
) {
    LifecycleResumeEffect(servoView) {
        servoView.servo.suspend(false)
        onPauseOrDispose { servoView.servo.suspend(true) }
    }
    LaunchedEffect(Unit) {
        while (true) {
            withFrameNanos { servoView.servo.onDoFrame() }
        }
    }
    LaunchedEffect(servoView.servo, servoView.navigator) {
        servoView.navigator.consumeNavigationEvents(servoView.servo)
    }

    val focusRequester = remember { FocusRequester() }
    AndroidView(
        factory = { _ -> servoView },
        modifier =
            modifier
                .focusRequester(focusRequester)
                .onKeyEvent { keyEvent ->
                    when (keyEvent.type) {
                        KeyEventType.KeyDown if keyEvent.key != Key.Back -> {
                            servoView.servo.onKeyDown(
                                keyEvent.key.nativeKeyCode,
                                keyEvent.nativeKeyEvent,
                            )
                            true
                        }

                        KeyEventType.KeyUp if keyEvent.key != Key.Back -> {
                            servoView.servo.onKeyUp(
                                keyEvent.key.nativeKeyCode,
                                keyEvent.nativeKeyEvent,
                            )
                            true
                        }

                        else -> false
                    }
                }
                .pointerInteropFilter(null) { motionEvent ->
                    focusRequester.requestFocus()

                    val action = motionEvent.actionMasked
                    val pointerIndex = motionEvent.actionIndex
                    val pointerId = motionEvent.getPointerId(pointerIndex)
                    val x = motionEvent.getX(pointerIndex)
                    val y = motionEvent.getY(pointerIndex)

                    when (action) {
                        MotionEvent.ACTION_DOWN,
                        MotionEvent.ACTION_POINTER_DOWN ->
                            servoView.servo.touchDown(x, y, pointerId)
                        MotionEvent.ACTION_MOVE -> servoView.servo.touchMove(x, y, pointerId)
                        MotionEvent.ACTION_UP,
                        MotionEvent.ACTION_POINTER_UP -> servoView.servo.touchUp(x, y, pointerId)
                        MotionEvent.ACTION_CANCEL -> servoView.servo.touchCancel(x, y, pointerId)
                    }

                    true
                },
        onRelease = { servoView.servo.glDispatcher.close() },
    )
}

@Stable
class ServoNavigator {
    private sealed interface NavigationEvent : Interaction {
        data class Navigate(val uri: String) : NavigationEvent

        data object Back : NavigationEvent

        data object Forward : NavigationEvent

        data object Reload : NavigationEvent

        data object Stop : NavigationEvent
    }

    private val coroutineScope = CoroutineScope(EmptyCoroutineContext)
    private val navigationEvents = MutableSharedFlow<NavigationEvent>()

    internal suspend fun consumeNavigationEvents(servo: Servo) {
        navigationEvents.collect { navigationEvent ->
            when (navigationEvent) {
                is NavigationEvent.Navigate -> servo.loadUri(navigationEvent.uri)
                NavigationEvent.Back -> servo.goBack()
                NavigationEvent.Forward -> servo.goForward()
                NavigationEvent.Reload -> servo.reload()
                NavigationEvent.Stop -> servo.stop()
            }
        }
    }

    var canGoBack by mutableStateOf(false)
        internal set

    var canGoForward by mutableStateOf(false)
        internal set

    fun navigate(uri: String) {
        coroutineScope.launch { navigationEvents.emit(NavigationEvent.Navigate(uri)) }
    }

    fun back() {
        coroutineScope.launch { navigationEvents.emit(NavigationEvent.Back) }
    }

    fun forward() {
        coroutineScope.launch { navigationEvents.emit(NavigationEvent.Forward) }
    }

    fun reload() {
        coroutineScope.launch { navigationEvents.emit(NavigationEvent.Reload) }
    }

    fun stop() {
        coroutineScope.launch { navigationEvents.emit(NavigationEvent.Stop) }
    }
}

class Servo(
    args: String?,
    url: String?,
    logStr: String?,
    experimentalMode: Boolean,
    private val scope: CoroutineScope,
    client: Client,
    context: Context,
    navigator: ServoNavigator,
) {
    private val jni = JNIServo()
    internal val glDispatcher = Executors.newSingleThreadExecutor().asCoroutineDispatcher()
    private val servoCallbacks = Callbacks(client, jni, scope, glDispatcher, navigator)

    init {
        scope.launch(glDispatcher) {
            jni.init(
                context,
                args,
                url,
                logStr,
                experimentalMode,
                servoCallbacks,
            )
        }
    }

    fun addPlatformWindow(
        size: Size,
        density: Float,
        surface: Surface,
    ) {
        scope.launch(glDispatcher) {
            jni.addPlatformWindow(
                size,
                density,
                surface,
            )
        }
    }

    fun resize(size: Size) {
        scope.launch(glDispatcher) { jni.resize(size) }
    }

    fun reload() {
        scope.launch(glDispatcher) { jni.reload() }
    }

    fun stop() {
        scope.launch(glDispatcher) { jni.stop() }
    }

    fun goBack() {
        scope.launch(glDispatcher) { jni.goBack() }
    }

    fun goForward() {
        scope.launch(glDispatcher) { jni.goForward() }
    }

    fun loadUri(uri: String) {
        scope.launch(glDispatcher) { jni.loadUri(uri) }
    }

    fun onKeyDown(keyCode: Int, event: KeyEvent) {
        scope.launch(glDispatcher) { jni.keydown(keyCode, event.unicodeChar) }
    }

    fun onKeyUp(keyCode: Int, event: KeyEvent) {
        scope.launch(glDispatcher) { jni.keyup(keyCode, event.unicodeChar) }
    }

    fun touchDown(x: Float, y: Float, pointerId: Int) {
        scope.launch(glDispatcher) { jni.touchDown(x, y, pointerId) }
    }

    fun touchMove(x: Float, y: Float, pointerId: Int) {
        scope.launch(glDispatcher) { jni.touchMove(x, y, pointerId) }
    }

    fun touchUp(x: Float, y: Float, pointerId: Int) {
        scope.launch(glDispatcher) { jni.touchUp(x, y, pointerId) }
    }

    fun touchCancel(x: Float, y: Float, pointerId: Int) {
        scope.launch(glDispatcher) { jni.touchCancel(x, y, pointerId) }
    }

    fun pausePainting() {
        scope.launch(glDispatcher) { jni.pausePainting() }
    }

    fun resumePainting(surface: Surface, size: Size) {
        scope.launch(glDispatcher) { jni.resumePainting(surface, size) }
    }

    fun suspend(suspended: Boolean) {
        servoCallbacks.suspended = suspended
    }

    fun mediaSessionAction(action: Int) {
        scope.launch(glDispatcher) { jni.mediaSessionAction(action) }
    }

    fun setExperimentalMode(enable: Boolean) {
        scope.launch(glDispatcher) { jni.setExperimentalMode(enable) }
    }

    fun onDoFrame() {
        scope.launch(glDispatcher) { jni.doFrame() }
    }

    interface Client {
        fun onAlert(message: String)

        fun onLoadStarted()

        fun onLoadEnded()

        fun onTitleChanged(title: String)

        fun onUrlChanged(url: String)

        fun onImeShow()

        fun onImeHide()

        fun onMediaSessionMetadata(title: String, artist: String, album: String)

        fun onMediaSessionPlaybackStateChange(state: Int)

        fun onMediaSessionSetPositionState(duration: Float, position: Float, playbackRate: Float)
    }

    private class Callbacks(
        private var client: Client,
        private val jni: JNIServo,
        private val scope: CoroutineScope,
        private val glDispatcher: CoroutineDispatcher,
        private val navigator: ServoNavigator,
    ) : JNIServo.Callbacks, Client {
        var suspended: Boolean = false

        override fun wakeup() {
            if (!suspended) {
                scope.launch(glDispatcher) { jni.performUpdates() }
            }
        }

        override fun onAlert(message: String) {
            scope.launch { client.onAlert(message) }
        }

        override fun onImeShow() {
            scope.launch { client.onImeShow() }
        }

        override fun onImeHide() {
            scope.launch { client.onImeHide() }
        }

        override fun onLoadStarted() {
            scope.launch { client.onLoadStarted() }
        }

        override fun onLoadEnded() {
            scope.launch { client.onLoadEnded() }
        }

        override fun onTitleChanged(title: String) {
            scope.launch { client.onTitleChanged(title) }
        }

        override fun onUrlChanged(url: String) {
            scope.launch { client.onUrlChanged(url) }
        }

        override fun onHistoryChanged(canGoBack: Boolean, canGoForward: Boolean) {
            navigator.canGoBack = canGoBack
            navigator.canGoForward = canGoForward
        }

        override fun onMediaSessionMetadata(title: String, artist: String, album: String) {
            scope.launch { client.onMediaSessionMetadata(title, artist, album) }
        }

        override fun onMediaSessionPlaybackStateChange(state: Int) {
            scope.launch { client.onMediaSessionPlaybackStateChange(state) }
        }

        override fun onMediaSessionSetPositionState(
            duration: Float,
            position: Float,
            playbackRate: Float,
        ) {
            scope.launch { client.onMediaSessionSetPositionState(duration, position, playbackRate) }
        }
    }
}
