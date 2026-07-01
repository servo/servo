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
import android.view.KeyEvent
import android.view.View
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputMethodManager
import android.widget.EditText
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.platform.ComposeView
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.core.content.getSystemService
import androidx.core.view.isVisible
import androidx.preference.PreferenceManager
import com.google.android.material.bottomnavigation.BottomNavigationView
import com.google.android.material.progressindicator.CircularProgressIndicator
import org.servo.servoview.Servo
import org.servo.servoview.ServoView

class MainActivity : AppCompatActivity(), Servo.Client {
    private lateinit var servoView: ServoView
    private var bottomNav: BottomNavigationView? = null

    private lateinit var urlField: EditText
    private var urlFieldIsFocused = false

    private lateinit var progressBar: CircularProgressIndicator
    private lateinit var idleText: TextView
    private var canGoBackState = mutableStateOf(false)
    private var canGoForwardState = mutableStateOf(false)
    private var isRefreshingState = mutableStateOf(false)
    private var mediaSession: MediaSession? = null
    private lateinit var historyManager: HistoryManager
    private var currentUrl = ""
    private var currentTitle = ""

    private class Settings(preferences: SharedPreferences) {
        var showAnimatingIndicator = preferences.getBoolean("animating_indicator", false)
        var experimental = preferences.getBoolean("experimental", false)
    }

    private lateinit var settings: Settings

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        servoView = findViewById(R.id.servoview)
        urlField = findViewById(R.id.urlfield)
        progressBar = findViewById(R.id.progressbar)
        idleText = findViewById(R.id.redrawing)

        historyManager = HistoryManager(this)

        updateSettingsIfNecessary(true)

        /*
        We use both Menu+MenuItems and Buttons for the same functions,
        depending on whether we’re in a phone or tablet+ layout. For the phone, we want
        the affordances of a navigation bar that uses a Menu (mBottomNav), but there’s no
        straightforward way to re-use these MenuItems to place them in the top toolbar
        in the tablet layout. The inverse approach has other problems. So we use
        - mBottomNav with a Menu + MenuItems on phones
        - individual Buttons added to the MaterialToolbar that also holds the URLInput on
          tablets and larger sizes
         */

        bottomNav = findViewById(R.id.bottom_bar)
        bottomNav?.setOnItemSelectedListener { item -> dispatchAction(item.itemId) }

        findViewById<View>(R.id.toolbar)?.apply {
            findViewById<ComposeView>(R.id.history_back_menu_item).apply {
                setContent {
                    IconButton(onClick = { dispatchAction(id) }, enabled = canGoBackState.value) {
                        Icon(painterResource(R.drawable.arrow_back), stringResource(R.string.history_back))
                    }
                }
            }
            findViewById<ComposeView>(R.id.history_forward_menu_item).apply {
                setContent {
                    IconButton(onClick = { dispatchAction(id) }, enabled = canGoForwardState.value) {
                        Icon(painterResource(R.drawable.arrow_forward), stringResource(R.string.history_forward))
                    }
                }
            }
            findViewById<ComposeView>(R.id.refresh_menu_item).apply {
                setContent {
                    IconButton(onClick = { dispatchAction(if (isRefreshingState.value) R.id.cancel_menu_item else R.id.refresh_menu_item) }) {
                        if (isRefreshingState.value) {
                            Icon(painterResource(R.drawable.cancel), stringResource(R.string.cancel))
                        } else {
                            Icon(painterResource(R.drawable.refresh), stringResource(R.string.refresh))
                        }
                    }
                }
            }
            findViewById<ComposeView>(R.id.settings_menu_item).apply {
                setContent {
                    IconButton(onClick = { dispatchAction(id) }) {
                        Icon(painterResource(R.drawable.settings), stringResource(R.string.options))
                    }
                }
            }
            findViewById<ComposeView>(R.id.history_menu_item).apply {
                setContent {
                    IconButton(onClick = { dispatchAction(id) }) {
                        Icon(painterResource(R.drawable.history), stringResource(R.string.history_title))
                    }
                }
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
        setupUrlField()
    }

    override fun onDestroy() {
        super.onDestroy()
        mediaSession?.hideMediaSessionControls()
    }

    // Handle UI actions (same handlers for MenuItems in phone layout
    // and View buttons in tablet layout
    private fun dispatchAction(id: Int): Boolean {
        when (id) {
            R.id.history_back_menu_item -> {
                // We’re unsetting all the loading UI just in case loading got stuck, and we’re
                // navigating to a cached page, which doesn’t trigger .onLoadEnded(). The "stop
                // loading" button is implemented (`cancel_menu_item`), but the underlying
                // Servo view can’t actually `stop()` yet.
                onLoadEnded()
                servoView.goBack()
            }
            R.id.history_forward_menu_item -> {
                // See above
                onLoadEnded()
                servoView.goForward()
            }
            R.id.refresh_menu_item -> {
                servoView.reload()
            }
            R.id.cancel_menu_item -> {
                servoView.stop()
            }
            R.id.settings_menu_item -> {
                startActivity(Intent(this, SettingsActivity::class.java))
            }
            R.id.history_menu_item -> {
                startActivityForResult(Intent(this, HistoryActivity::class.java), HISTORY_REQUEST_CODE)
            }
        }
        return false
    }

    private fun setupUrlField() {
        urlField.setOnEditorActionListener { _, actionId, _ ->
            if (actionId == EditorInfo.IME_ACTION_DONE) {
                loadUrlFromField()
                servoView.requestFocus()
                true
            } else {
                false
            }
        }
        urlField.setOnFocusChangeListener { v, hasFocus ->
            if (v.id == R.id.urlfield) {
                urlFieldIsFocused = hasFocus
                if (!hasFocus) {
                    getSystemService<InputMethodManager>()?.hideSoftInputFromWindow(v.windowToken, 0)
                }
            }
        }
    }

    private fun loadUrlFromField() {
        servoView.loadUri(urlField.getText().toString().trim { it <= ' ' })
    }

    override fun onImeShow() {
        getSystemService<InputMethodManager>()?.showSoftInput(servoView, InputMethodManager.SHOW_IMPLICIT)
    }

    override fun onImeHide() {
        getSystemService<InputMethodManager>()?.hideSoftInputFromWindow(servoView.windowToken, InputMethodManager.SHOW_IMPLICIT)
    }

    override fun onKeyDown(keyCode: Int, event: KeyEvent?): Boolean {
        if (urlFieldIsFocused) {
            return true
        }
        return servoView.onKeyDown(keyCode, event)
    }

    override fun onKeyUp(keyCode: Int, event: KeyEvent?): Boolean {
        if (urlFieldIsFocused) {
            return true
        }
        return servoView.onKeyUp(keyCode, event)
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
        // Phone view
        bottomNav?.let { bottomNav ->
            bottomNav.menu.findItem(R.id.cancel_menu_item).isVisible = true
            bottomNav.menu.findItem(R.id.refresh_menu_item).isVisible = false
        }
        isRefreshingState.value = true

        progressBar.isVisible = true
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
        // Phone view
        bottomNav?.let { bottomNav ->
            bottomNav.menu.findItem(R.id.cancel_menu_item).isVisible = false
            bottomNav.menu.findItem(R.id.refresh_menu_item).isVisible = true
        }
        isRefreshingState.value = false
        progressBar.isVisible = false
    }

    override fun onTitleChanged(title: String?) {
        currentTitle = title.orEmpty()
    }

    override fun onUrlChanged(url: String?) {
        urlField.setText(url)
        currentUrl = url.orEmpty()
    }

    override fun onHistoryChanged(canGoBack: Boolean, canGoForward: Boolean) {
        Log.i(TAG, "onHistoryChanged: $canGoBack<->$canGoForward")
        // Phone view
        bottomNav?.let { bottomNav ->
            bottomNav.menu.findItem(R.id.history_back_menu_item).isEnabled = canGoBack
            bottomNav.menu.findItem(R.id.history_forward_menu_item).isEnabled = canGoForward
        }
        canGoBackState.value = canGoBack
        canGoForwardState.value = canGoForward
    }

    override fun onRedrawing(redrawing: Boolean) {
        idleText.setText(if (redrawing) R.string.loop else R.string.idle)
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
                urlField.setText(url)
                loadUrlFromField()
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

    private fun onAnimatingIndicatorPrefChanged(value: Boolean) {
        idleText.isVisible = value
    }

    private fun onExperimentalPrefChanged(value: Boolean) {
        servoView.setExperimentalMode(value)
    }

    private fun updateSettingsIfNecessary(force: Boolean) {
        val preferences = PreferenceManager.getDefaultSharedPreferences(applicationContext)
        val updated = Settings(preferences)

        if (force || updated.showAnimatingIndicator != settings.showAnimatingIndicator) {
            onAnimatingIndicatorPrefChanged(updated.showAnimatingIndicator)
        }

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
