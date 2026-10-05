/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
package org.servo.servoshell

import android.content.Intent
import android.content.SharedPreferences
import android.os.Bundle
import android.system.ErrnoException
import android.system.Os
import android.util.Log
import android.view.inputmethod.InputMethodManager
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.setContent
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.component1
import androidx.activity.result.component2
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.selectAll
import androidx.compose.material3.AlertDialog
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
import androidx.compose.material3.TextButton
import androidx.compose.material3.adaptive.currentWindowAdaptiveInfo
import androidx.compose.material3.rememberSearchBarState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.core.content.getSystemService
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LifecycleEventEffect
import androidx.lifecycle.lifecycleScope
import androidx.preference.PreferenceManager
import androidx.window.core.layout.WindowSizeClass
import kotlinx.coroutines.launch
import org.servo.servoview.Servo
import org.servo.servoview.ServoNavigator
import org.servo.servoview.ServoView

class MainActivity : ComponentActivity(), Servo.Client {
    private lateinit var servoView: ServoView

    private val urlTextFieldState = TextFieldState()
    private var isRefreshing by mutableStateOf(false)
    private lateinit var mediaSession: MediaSession
    private lateinit var historyManager: HistoryManager
    private var currentUrl = ""
    private var currentTitle = ""
    private var alertMessage by mutableStateOf<String?>(null)

    private class Settings(preferences: SharedPreferences) {
        var experimental = preferences.getBoolean("experimental", false)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val sharedPreferences = PreferenceManager.getDefaultSharedPreferences(applicationContext)
        var settings = Settings(sharedPreferences)
        val navigator = ServoNavigator()
        servoView =
            ServoView(
                context = this,
                client = this,
                servoArgs = intent.getStringExtra("servoargs"),
                servoLog = intent.getStringExtra("servolog"),
                experimentalMode = settings.experimental,
                initialUri =
                    if (Intent.ACTION_VIEW == intent.action) intent.data.toString() else null,
                navigator = navigator,
                scope = lifecycleScope,
            )

        mediaSession = MediaSession(servoView, applicationContext)
        historyManager = HistoryManager(this)

        val historyActivityResultLauncher =
            registerForActivityResult(ActivityResultContracts.StartActivityForResult()) {
                (resultCode, data) ->
                if (resultCode == RESULT_OK && data != null) {
                    val url = data.getStringExtra("url")
                    if (!url.isNullOrEmpty()) {
                        urlTextFieldState.edit { replace(0, length, url) }
                        navigator.navigate(urlTextFieldState.text.toString())
                    }
                }
            }

        setContent {
            LifecycleEventEffect(Lifecycle.Event.ON_RESUME) {
                val updatedSettings = Settings(sharedPreferences)
                if (updatedSettings.experimental != settings.experimental) {
                    servoView.setExperimentalMode(updatedSettings.experimental)
                }
                settings = updatedSettings
            }

            val isWindowWidthAtLeastMedium =
                currentWindowAdaptiveInfo()
                    .windowSizeClass
                    .isWidthAtLeastBreakpoint(WindowSizeClass.WIDTH_DP_MEDIUM_LOWER_BOUND)

            val servoFocusRequester = remember { FocusRequester() }
            Scaffold(
                topBar = {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        if (isWindowWidthAtLeastMedium) {
                            IconButton(
                                onClick = { onHistoryBackMenuItemClicked(navigator) },
                                enabled = navigator.canGoBack,
                            ) {
                                Icon(
                                    painterResource(R.drawable.arrow_back),
                                    stringResource(R.string.history_back),
                                )
                            }
                            IconButton(
                                onClick = { onHistoryForwardMenuItemClicked(navigator) },
                                enabled = navigator.canGoForward,
                            ) {
                                Icon(
                                    painterResource(R.drawable.arrow_forward),
                                    stringResource(R.string.history_forward),
                                )
                            }
                            IconButton(
                                onClick = {
                                    if (isRefreshing) onCancelMenuItemClicked(navigator)
                                    else onRefreshMenuItemClicked(navigator)
                                }
                            ) {
                                if (isRefreshing) {
                                    Icon(
                                        painterResource(R.drawable.cancel),
                                        stringResource(R.string.cancel),
                                    )
                                } else {
                                    Icon(
                                        painterResource(R.drawable.refresh),
                                        stringResource(R.string.refresh),
                                    )
                                }
                            }
                        }
                        Omnibox(
                            urlTextFieldState,
                            onSearch = { search ->
                                navigator.navigate(search)
                                servoFocusRequester.requestFocus()
                            },
                            modifier = Modifier.weight(1f).padding(end = 10.dp),
                        )
                        if (isRefreshing) {
                            CircularProgressIndicator(
                                modifier = Modifier.padding(end = 10.dp).size(20.dp)
                            )
                        }
                        if (isWindowWidthAtLeastMedium) {
                            IconButton(onClick = ::onSettingsMenuItemClicked) {
                                Icon(
                                    painterResource(R.drawable.settings),
                                    stringResource(R.string.options),
                                )
                            }
                            IconButton(
                                onClick = {
                                    onHistoryMenuItemClicked(historyActivityResultLauncher)
                                }
                            ) {
                                Icon(
                                    painterResource(R.drawable.history),
                                    stringResource(R.string.history_title),
                                )
                            }
                        }
                    }
                },
                bottomBar = {
                    if (!isWindowWidthAtLeastMedium) {
                        NavigationBar {
                            NavigationBarItem(
                                selected = false,
                                enabled = navigator.canGoBack,
                                onClick = { onHistoryBackMenuItemClicked(navigator) },
                                icon = { Icon(painterResource(R.drawable.arrow_back), null) },
                                label = { Text(stringResource(R.string.history_back)) },
                            )
                            NavigationBarItem(
                                selected = false,
                                enabled = navigator.canGoForward,
                                onClick = { onHistoryForwardMenuItemClicked(navigator) },
                                icon = { Icon(painterResource(R.drawable.arrow_forward), null) },
                                label = { Text(stringResource(R.string.history_forward)) },
                            )
                            if (isRefreshing) {
                                NavigationBarItem(
                                    selected = false,
                                    onClick = { onCancelMenuItemClicked(navigator) },
                                    icon = { Icon(painterResource(R.drawable.cancel), null) },
                                    label = { Text(stringResource(R.string.cancel)) },
                                )
                            } else {
                                NavigationBarItem(
                                    selected = false,
                                    onClick = { onRefreshMenuItemClicked(navigator) },
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
                                onClick = {
                                    onHistoryMenuItemClicked(historyActivityResultLauncher)
                                },
                                icon = { Icon(painterResource(R.drawable.history), null) },
                                label = { Text(stringResource(R.string.history_title)) },
                            )
                        }
                    }
                },
            ) { innerPadding ->
                Servo(
                    servoView = servoView,
                    modifier = Modifier.padding(innerPadding).focusRequester(servoFocusRequester),
                )
                BackHandler(enabled = navigator.canGoBack) { navigator.back() }
                LaunchedEffect(servoFocusRequester) { servoFocusRequester.requestFocus() }
                alertMessage?.let { alertMessage ->
                    AlertDialog(
                        onDismissRequest = { this.alertMessage = null },
                        confirmButton = {
                            TextButton(onClick = { this.alertMessage = null }) {
                                Text(stringResource(android.R.string.ok))
                            }
                        },
                        text = { Text(alertMessage) },
                    )
                }
            }
        }

        val sdcard = getExternalFilesDir("")
        val host = sdcard!!.toPath().resolve("android_hosts").toString()
        try {
            Os.setenv("HOST_FILE", host, false)
        } catch (e: ErrnoException) {
            e.printStackTrace()
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        mediaSession.hideMediaSessionControls()
    }

    private fun onHistoryBackMenuItemClicked(navigator: ServoNavigator) {
        navigator.back()
    }

    private fun onHistoryForwardMenuItemClicked(navigator: ServoNavigator) {
        navigator.forward()
    }

    private fun onRefreshMenuItemClicked(navigator: ServoNavigator) {
        navigator.reload()
    }

    private fun onCancelMenuItemClicked(navigator: ServoNavigator) {
        navigator.stop()
    }

    private fun onSettingsMenuItemClicked() {
        startActivity(Intent(this, SettingsActivity::class.java))
    }

    private fun onHistoryMenuItemClicked(
        historyActivityResultLauncher: ActivityResultLauncher<Intent>
    ) {
        historyActivityResultLauncher.launch(Intent(this, HistoryActivity::class.java))
    }

    override fun onImeShow() {
        getSystemService<InputMethodManager>()
            ?.showSoftInput(servoView, InputMethodManager.SHOW_IMPLICIT)
    }

    override fun onImeHide() {
        getSystemService<InputMethodManager>()
            ?.hideSoftInputFromWindow(servoView.windowToken, InputMethodManager.HIDE_IMPLICIT_ONLY)
    }

    override fun onAlert(message: String) {
        alertMessage = message
    }

    override fun onLoadStarted() {
        // This doesn’t seem to actually happen when navigating
        // back to a page that is already cached.
        Log.i(TAG, "onLoadStarted: ")
        isRefreshing = true
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
        isRefreshing = false
    }

    override fun onTitleChanged(title: String) {
        currentTitle = title
    }

    override fun onUrlChanged(url: String) {
        urlTextFieldState.edit { replace(0, length, url) }
        currentUrl = url
    }

    override fun onMediaSessionMetadata(title: String, artist: String, album: String) {
        Log.d("onMediaSessionMetadata", "$title $artist $album")
        mediaSession.updateMetadata(title, artist, album)
    }

    override fun onMediaSessionPlaybackStateChange(state: Int) {
        Log.d("onMediaSessionPlaybackStateChange", state.toString())
        mediaSession.setPlaybackState(state)

        if (state == MediaSession.PLAYBACK_STATE_NONE) {
            mediaSession.hideMediaSessionControls()
            return
        }
        if (
            state == MediaSession.PLAYBACK_STATE_PLAYING ||
                state == MediaSession.PLAYBACK_STATE_PAUSED
        ) {
            mediaSession.showMediaSessionControls()
        }
    }

    override fun onMediaSessionSetPositionState(
        duration: Float,
        position: Float,
        playbackRate: Float,
    ) {
        Log.d("onMediaSessionSetPositionState", "$duration $position $playbackRate")
    }

    companion object {
        private const val TAG = "MainActivity"
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
                modifier =
                    Modifier.onFocusChanged { focusState ->
                        if (focusState.isFocused) {
                            coroutineScope.launch { textFieldState.edit { selectAll() } }
                        }
                    },
                placeholder = { Text(stringResource(R.string.url_or_search)) },
            )
        },
        modifier = modifier,
    )
}
