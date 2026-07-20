/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
package org.servo.servoshell

import android.app.AlertDialog
import android.content.Intent
import android.content.SharedPreferences
import android.os.Bundle
import android.system.ErrnoException
import android.system.Os
import android.util.Log
import android.view.inputmethod.InputMethodManager
import androidx.activity.compose.setContent
import androidx.appcompat.app.AppCompatActivity
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.selectAll
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SearchBar
import androidx.compose.material3.SearchBarDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.adaptive.currentWindowAdaptiveInfo
import androidx.compose.material3.rememberSearchBarState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.content.getSystemService
import androidx.preference.PreferenceManager
import kotlinx.coroutines.launch
import org.servo.servoview.Servo
import org.servo.servoview.ServoView

class MainActivity : AppCompatActivity(), Servo.Client {
    private lateinit var servoView: ServoView

    private val urlTextFieldState = TextFieldState()
    private var canGoBackState = mutableStateOf(false)
    private var canGoForwardState = mutableStateOf(false)
    private var isRefreshingState = mutableStateOf(false)
    private var mediaSession: MediaSession? = null
    private lateinit var historyManager: HistoryManager
    private var currentUrl = ""
    private var currentTitle = ""

    private class Settings(preferences: SharedPreferences) {
        var experimental = preferences.getBoolean("experimental", false)
    }

    private lateinit var settings: Settings

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        servoView = ServoView(this)

        historyManager = HistoryManager(this)

        updateSettingsIfNecessary(true)

        setContent {
            val isWindowWidthAtLeastMedium = currentWindowAdaptiveInfo().windowSizeClass.isWidthAtLeastBreakpoint(600)

            Scaffold(
                topBar = {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        if (isWindowWidthAtLeastMedium) {
                            IconButton(onClick = ::onHistoryBackMenuItemClicked, enabled = canGoBackState.value) {
                                Icon(painterResource(R.drawable.arrow_back), stringResource(R.string.history_back))
                            }
                            IconButton(onClick = ::onHistoryForwardMenuItemClicked, enabled = canGoForwardState.value) {
                                Icon(painterResource(R.drawable.arrow_forward), stringResource(R.string.history_forward))
                            }
                            IconButton(onClick = { if (isRefreshingState.value) onCancelMenuItemClicked() else onRefreshMenuItemClicked() }) {
                                if (isRefreshingState.value) {
                                    Icon(painterResource(R.drawable.cancel), stringResource(R.string.cancel))
                                } else {
                                    Icon(painterResource(R.drawable.refresh), stringResource(R.string.refresh))
                                }
                            }
                        }
                        Omnibox(
                            urlTextFieldState,
                            onSearch = { search ->
                                loadUrl(search)
                                servoView.requestFocus()
                            },
                            modifier = Modifier
                                .weight(1f)
                                .padding(end = 10.dp),
                        )
                        if (isRefreshingState.value) {
                            CircularProgressIndicator(
                                modifier = Modifier
                                    .padding(end = 10.dp)
                                    .size(20.dp),
                            )
                        }
                        if (isWindowWidthAtLeastMedium) {
                            IconButton(onClick = ::onSettingsMenuItemClicked) {
                                Icon(painterResource(R.drawable.settings), stringResource(R.string.options))
                            }
                            IconButton(onClick = ::onHistoryMenuItemClicked) {
                                Icon(painterResource(R.drawable.history), stringResource(R.string.history_title))
                            }
                        }
                    }
                },
                bottomBar = {
                    if (!isWindowWidthAtLeastMedium) {
                        NavigationBar {
                            NavigationBarItem(
                                selected = false,
                                enabled = canGoBackState.value,
                                onClick = ::onHistoryBackMenuItemClicked,
                                icon = { Icon(painterResource(R.drawable.arrow_back), null) },
                                label = { Text(stringResource(R.string.history_back)) },
                            )
                            NavigationBarItem(
                                selected = false,
                                enabled = canGoForwardState.value,
                                onClick = { onHistoryForwardMenuItemClicked() },
                                icon = { Icon(painterResource(R.drawable.arrow_forward), null) },
                                label = { Text(stringResource(R.string.history_forward)) },
                            )
                            if (isRefreshingState.value) {
                                NavigationBarItem(
                                    selected = false,
                                    onClick = ::onCancelMenuItemClicked,
                                    icon = { Icon(painterResource(R.drawable.cancel), null) },
                                    label = { Text(stringResource(R.string.cancel)) },
                                )
                            } else {
                                NavigationBarItem(
                                    selected = false,
                                    onClick = ::onRefreshMenuItemClicked,
                                    icon = { Icon(painterResource(R.drawable.refresh), null) },
                                    label = { Text(stringResource(R.string.refresh)) },
                                )
                            }
                            NavigationBarItem(
                                selected = false,
                                onClick = ::onSettingsMenuItemClicked,
                                icon = { Icon(painterResource(R.drawable.settings), null) },
                                label = { Text(stringResource(R.string.options)) },
                            )
                            NavigationBarItem(
                                selected = false,
                                onClick = ::onHistoryMenuItemClicked,
                                icon = { Icon(painterResource(R.drawable.history), null) },
                                label = { Text(stringResource(R.string.history_title)) },
                            )
                        }
                    }
                },
            ) { innerPadding ->
                AndroidView(
                    factory = { _ -> servoView },
                    modifier = Modifier.padding(innerPadding),
                )
            }
        }

        servoView.setClient(this)
        servoView.requestFocus()

        val sdcard = getExternalFilesDir("")
        val host = sdcard!!.toPath().resolve("android_hosts").toString()
        try {
            Os.setenv("HOST_FILE", host, false)
        } catch (e: ErrnoException) {
            e.printStackTrace()
        }

        val intent = getIntent()
        val args = intent.getStringExtra("servoargs")
        val log = intent.getStringExtra("servolog")
        servoView.setServoArgs(args, log, settings.experimental)

        if (Intent.ACTION_VIEW == intent.action) {
            servoView.loadUri(intent.data.toString())
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        mediaSession?.hideMediaSessionControls()
    }

    /**
     * We’re unsetting all the loading UI just in case loading got stuck, and we’re
     * navigating to a cached page, which doesn’t trigger [onLoadEnded]. The "stop
     * loading" button is implemented by [onCancelMenuItemClicked], but the underlying
     * Servo view can’t actually [ServoView.stop] yet.
     */
    private fun onHistoryItemClicked() {
        onLoadEnded()
    }

    private fun onHistoryBackMenuItemClicked() {
        onHistoryItemClicked()
        servoView.goBack()
    }

    private fun onHistoryForwardMenuItemClicked() {
        onHistoryItemClicked()
        servoView.goForward()
    }

    private fun onRefreshMenuItemClicked() {
        servoView.reload()
    }

    private fun onCancelMenuItemClicked() {
        servoView.stop()
    }

    private fun onSettingsMenuItemClicked() {
        startActivity(Intent(this, SettingsActivity::class.java))
    }

    private fun onHistoryMenuItemClicked() {
        startActivityForResult(Intent(this, HistoryActivity::class.java), HISTORY_REQUEST_CODE)
    }

    private fun loadUrl(search: String) {
        servoView.loadUri(search.trim { it <= ' ' })
    }

    override fun onImeShow() {
        getSystemService<InputMethodManager>()?.showSoftInput(servoView, InputMethodManager.SHOW_IMPLICIT)
    }

    override fun onImeHide() {
        getSystemService<InputMethodManager>()?.hideSoftInputFromWindow(servoView.windowToken, InputMethodManager.SHOW_IMPLICIT)
    }

    override fun onAlert(message: String?) {
        AlertDialog.Builder(this)
            .setMessage(message)
            .show()
    }

    override fun onLoadStarted() {
        // This doesn’t seem to actually happen when navigating
        // back to a page that is already cached.
        Log.i(TAG, "onLoadStarted: ")
        isRefreshingState.value = true
    }

    // INFO: This currently gets called multiple times on each load.
    override fun onLoadEnded() {
        Log.i(TAG, "onLoadEnded: ")
        if (currentUrl.isNotEmpty()) {
            // HistoryManager has a basic method of preventing clobbering
            // by the fact that onLoadEnded gets called multiple times
            // per page. 
            historyManager.addEntry(currentUrl, currentTitle)
        }
        isRefreshingState.value = false
    }

    override fun onTitleChanged(title: String?) {
        currentTitle = title.orEmpty()
    }

    override fun onUrlChanged(url: String?) {
        val url = url.orEmpty()
        urlTextFieldState.edit { replace(0, length, url) }
        currentUrl = url
    }

    override fun onHistoryChanged(canGoBack: Boolean, canGoForward: Boolean) {
        Log.i(TAG, "onHistoryChanged: $canGoBack<->$canGoForward")
        canGoBackState.value = canGoBack
        canGoForwardState.value = canGoForward
    }

    override fun onRedrawing(redrawing: Boolean) {
    }

    public override fun onPause() {
        servoView.onPause()
        super.onPause()
    }

    public override fun onResume() {
        servoView.onResume()
        super.onResume()
        updateSettingsIfNecessary(false)
    }

    @Deprecated("Deprecated in Java")
    override fun onBackPressed() {
        if (canGoBackState.value) {
            servoView.goBack()
        } else {
            super.onBackPressed()
        }
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)

        if (requestCode == HISTORY_REQUEST_CODE && resultCode == RESULT_OK && data != null) {
            val url = data.getStringExtra("url")
            if (!url.isNullOrEmpty()) {
                urlTextFieldState.edit { replace(0, length, url) }
                loadUrl(urlTextFieldState.text.toString())
            }
        }
    }

    override fun onMediaSessionMetadata(title: String?, artist: String?, album: String?) {
        Log.d("onMediaSessionMetadata", "$title $artist $album")
        val mediaSession = mediaSession ?: MediaSession(servoView, applicationContext).also { mediaSession = it }
        mediaSession.updateMetadata(title, artist, album)
    }

    override fun onMediaSessionPlaybackStateChange(state: Int) {
        Log.d("onMediaSessionPlaybackStateChange", state.toString())
        val mediaSession = mediaSession ?: MediaSession(servoView, applicationContext).also { mediaSession = it }

        mediaSession.setPlaybackState(state)

        if (state == MediaSession.PLAYBACK_STATE_NONE) {
            mediaSession.hideMediaSessionControls()
            return
        }
        if (state == MediaSession.PLAYBACK_STATE_PLAYING ||
            state == MediaSession.PLAYBACK_STATE_PAUSED
        ) {
            mediaSession.showMediaSessionControls()
        }
    }

    override fun onMediaSessionSetPositionState(duration: Float, position: Float, playbackRate: Float) {
        Log.d("onMediaSessionSetPositionState", "$duration $position $playbackRate")
    }

    private fun onExperimentalPrefChanged(value: Boolean) {
        servoView.setExperimentalMode(value)
    }

    private fun updateSettingsIfNecessary(force: Boolean) {
        val preferences = PreferenceManager.getDefaultSharedPreferences(applicationContext)
        val updated = Settings(preferences)

        if (force || updated.experimental != settings.experimental) {
            onExperimentalPrefChanged(updated.experimental)
        }

        settings = updated
    }

    companion object {
        private const val TAG = "MainActivity"

        // Identify which activity a result came from, if we ever have more
        // than one
        private const val HISTORY_REQUEST_CODE = 1
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun Omnibox(
    textFieldState: TextFieldState,
    onSearch: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val searchBarState = rememberSearchBarState()

    SearchBar(
        state = searchBarState,
        inputField = {
            val coroutineScope = rememberCoroutineScope()

            SearchBarDefaults.InputField(
                textFieldState = textFieldState,
                searchBarState = searchBarState,
                onSearch = onSearch,
                modifier = Modifier
                    .onFocusChanged { focusState ->
                        if (focusState.isFocused) {
                            coroutineScope.launch {
                                textFieldState.edit { selectAll() }
                            }
                        }
                    },
                placeholder = { Text(stringResource(R.string.url_or_search)) },
            )
        },
        modifier = modifier,
    )
}
