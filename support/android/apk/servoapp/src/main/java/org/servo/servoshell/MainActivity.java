/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

package org.servo.servoshell;

import android.app.Activity;
import android.app.AlertDialog;
import android.content.Intent;
import android.content.SharedPreferences;
import android.os.Bundle;
import android.system.ErrnoException;
import android.system.Os;
import android.util.Log;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputMethodManager;
import android.view.KeyEvent;
import android.view.View;
import android.widget.EditText;
import android.widget.TextView;
import androidx.preference.PreferenceManager;

import com.google.android.material.bottomnavigation.BottomNavigationView;
import com.google.android.material.progressindicator.CircularProgressIndicator;

import org.servo.servoview.Servo;
import org.servo.servoview.ServoView;

import java.io.File;

public class MainActivity extends Activity implements Servo.Client {

    private static final String TAG = "MainActivity";
    // Identify which activity a result came from, if we ever have more
    // than one
    private static final int HISTORY_REQUEST_CODE = 1;

    private ServoView mServoView;
    private BottomNavigationView mBottomNav;

    private EditText mUrlField;
    private boolean mUrlFieldIsFocused;

    private CircularProgressIndicator mProgressBar;
    private TextView mIdleText;
    private boolean mCanGoBack;
    private MediaSession mMediaSession;
    private HistoryManager mHistoryManager;
    private String mCurrentUrl;
    private String mCurrentTitle;
    
    private static class Settings {
        Settings(SharedPreferences preferences) {
            showAnimatingIndicator = preferences.getBoolean("animating_indicator", false);
            experimental = preferences.getBoolean("experimental", false);
        }

        boolean showAnimatingIndicator;
        boolean experimental;
    }
    private Settings mSettings;

    private final View.OnClickListener actionClickListener = v -> dispatchAction(v.getId());

    // Binds a click listener to a View if it exists.
    // Useful for handling buttons that only exist in the tablet+ layout
    private void bindClick(int id) {
        View v = findViewById(id);
        if (v != null) {
            v.setOnClickListener(actionClickListener);
        }
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        mServoView = findViewById(R.id.servoview);
        mUrlField = findViewById(R.id.urlfield);
        mUrlFieldIsFocused = false;
        mProgressBar = findViewById(R.id.progressbar);
        mIdleText = findViewById(R.id.redrawing);
        mCanGoBack = false;
        
        // Initialize history manager
        mHistoryManager = new HistoryManager(this);
        // Since there doesn't seem to be a way to get the current URL
        // or title from the Servo view other than through 
        // the corresponding event handlers (e.g. onTitleChanged),
        // we need to keep them somewhere.
        mCurrentUrl = "";
        mCurrentTitle = "";

        updateSettingsIfNecessary(true);

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

        // Bind handlers to menu items (phone layout)
        mBottomNav = findViewById(R.id.bottom_bar);
        if (mBottomNav != null) {
            mBottomNav.setOnItemSelectedListener(item ->
                dispatchAction(item.getItemId())
            );
        }

        // Bind handlers to buttons, if they exist (tablet layout)
        bindClick(R.id.history_back_menu_item);
        bindClick(R.id.history_forward_menu_item);
        bindClick(R.id.refresh_menu_item);
        bindClick(R.id.cancel_menu_item);
        bindClick(R.id.settings_menu_item);
        bindClick(R.id.history_menu_item);

        mServoView.setClient(this);
        mServoView.requestFocus();

        File sdcard = getExternalFilesDir("");
        String host = sdcard.toPath().resolve("android_hosts").toString();
        try {
            Os.setenv("HOST_FILE", host, false);
        } catch (ErrnoException e) {
            e.printStackTrace();
        }

        Intent intent = getIntent();
        String args = intent.getStringExtra("servoargs");
        String log = intent.getStringExtra("servolog");
        mServoView.setServoArgs(args, log, mSettings.experimental);

        if (Intent.ACTION_VIEW.equals(intent.getAction())) {
            mServoView.loadUri(intent.getData().toString());
        }
        setupUrlField();
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        if (mMediaSession != null) {
            mMediaSession.hideMediaSessionControls();
        }
    }

    // Handle UI actions (same handlers for MenuItems in phone layout
    // and View buttons in tablet layout
    private boolean dispatchAction(int id) {
        if (id == R.id.history_back_menu_item) {
            // We’re unsetting all the loading UI just in case loading got stuck, and we’re
            // navigating to a cached page, which doesn’t trigger .onLoadEnded(). The "stop
            // loading" button is implemented (`cancel_menu_item`), but the underlying
            // Servo view can’t actually `stop()` yet.
            this.onLoadEnded();
            mServoView.goBack();
        } else if (id == R.id.history_forward_menu_item) {
            // See above
            this.onLoadEnded();
            mServoView.goForward();
        } else if (id == R.id.refresh_menu_item) {
            mServoView.reload();
        } else if (id == R.id.cancel_menu_item) {
            // stop() isn’t actually implemented yet.
            mServoView.stop();
        } else if (id == R.id.settings_menu_item) {
            Intent myIntent = new Intent(this, SettingsActivity.class);
            startActivity(myIntent);
        } else if (id == R.id.history_menu_item) {
            Intent myIntent = new Intent(this, HistoryActivity.class);
            startActivityForResult(myIntent, HISTORY_REQUEST_CODE);
        }
        return false;
    }

    private void setupUrlField() {
        mUrlField.setOnEditorActionListener((v, actionId, event) -> {
            if (actionId == EditorInfo.IME_ACTION_DONE) {
                loadUrlFromField();
                mServoView.requestFocus();
                return true;
            }
            return false;
        });
        mUrlField.setOnFocusChangeListener((v, hasFocus) -> {
            if (v.getId() == R.id.urlfield) {
                mUrlFieldIsFocused = hasFocus;
                if (!hasFocus) {
                    InputMethodManager imm = getSystemService(InputMethodManager.class);
                    imm.hideSoftInputFromWindow(v.getWindowToken(), 0);
                }
            }
        });
    }

    private void loadUrlFromField() {
        String text = mUrlField.getText().toString();
        text = text.trim();

        mServoView.loadUri(text);
    }

    @Override
    public void onImeShow() {
        InputMethodManager imm = getSystemService(InputMethodManager.class);
        imm.showSoftInput(mServoView, InputMethodManager.SHOW_IMPLICIT);
    }

    @Override
    public void onImeHide() {
        InputMethodManager imm = getSystemService(InputMethodManager.class);
        imm.hideSoftInputFromWindow(mServoView.getWindowToken(), InputMethodManager.SHOW_IMPLICIT);
    }

    @Override
    public boolean onKeyDown(int keyCode, KeyEvent event) {
        if (mUrlFieldIsFocused) {
            return true;
        }
        return mServoView.onKeyDown(keyCode, event);
    }

    @Override
    public boolean onKeyUp(int keyCode, KeyEvent event) {
        if (mUrlFieldIsFocused) {
            return true;
        }
        return mServoView.onKeyUp(keyCode, event);
    }

    @Override
    public void onAlert(String message) {
        AlertDialog.Builder builder = new AlertDialog.Builder(this);
        builder.setMessage(message);
        AlertDialog alert = builder.create();
        alert.show();
    }

    @Override
    public void onLoadStarted() {
        // This doesn’t seem to actually happen when navigating
        // back to a page that is already cached.
        Log.i(TAG, "onLoadStarted: ");
        // Phone view
        if (mBottomNav != null) {
            mBottomNav.getMenu().findItem(R.id.cancel_menu_item).setVisible(true);
            mBottomNav.getMenu().findItem(R.id.refresh_menu_item).setVisible(false);
        }
        // tablet view
        findViewById(R.id.cancel_menu_item).setVisibility(View.VISIBLE);
        findViewById(R.id.refresh_menu_item).setVisibility(View.GONE);

        mProgressBar.setVisibility(View.VISIBLE);
    }

    // INFO: This currently gets called multiple times on each load.
    @Override
    public void onLoadEnded() {
        Log.i(TAG, "onLoadEnded: ");
        // Add to history when a page starts loading
        if (mHistoryManager != null && mCurrentUrl != null && !mCurrentUrl.isEmpty()) {
            // mHistoryManager has a basic method of preventing clobbering
            // by the fact that onLoadEnded gets called multiple times
            // per page. 
            mHistoryManager.addEntry(mCurrentUrl, mCurrentTitle);
        }
        // Phone view
        if (mBottomNav != null) {
            mBottomNav.getMenu().findItem(R.id.cancel_menu_item).setVisible(false);
            mBottomNav.getMenu().findItem(R.id.refresh_menu_item).setVisible(true);
        }
        // tablet view
        findViewById(R.id.cancel_menu_item).setVisibility(View.GONE);
        findViewById(R.id.refresh_menu_item).setVisibility(View.VISIBLE);
        mProgressBar.setVisibility(View.GONE);
    }

    @Override
    public void onTitleChanged(String title) {
        mCurrentTitle = title != null ? title : "";
    }

    @Override
    public void onUrlChanged(String url) {
        mUrlField.setText(url);
        mCurrentUrl = url != null ? url : "";
    }

    @Override
    public void onHistoryChanged(boolean canGoBack, boolean canGoForward) {
        Log.i(TAG, "onHistoryChanged: " + canGoBack + "<->" + canGoForward);
        // Phone view
        if (mBottomNav != null) {
            mBottomNav.getMenu().findItem(R.id.history_back_menu_item).setEnabled(canGoBack);
            mBottomNav.getMenu().findItem(R.id.history_forward_menu_item).setEnabled(canGoForward);
        }
        // tablet view
        findViewById(R.id.history_back_menu_item).setEnabled(canGoBack);
        findViewById(R.id.history_forward_menu_item).setEnabled(canGoForward);
        mCanGoBack = canGoBack;
    }

    public void onRedrawing(boolean redrawing) {
        if (redrawing) {
            mIdleText.setText(R.string.loop);
        } else {
            mIdleText.setText(R.string.idle);
        }
    }

    @Override
    public void onPause() {
        mServoView.onPause();
        super.onPause();
    }

    @Override
    public void onResume() {
        mServoView.onResume();
        super.onResume();
        updateSettingsIfNecessary(false);
    }

    @Override
    public void onBackPressed() {
        if (mCanGoBack) {
            mServoView.goBack();
        } else {
            super.onBackPressed();
        }
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        
        if (requestCode == HISTORY_REQUEST_CODE && resultCode == RESULT_OK && data != null) {
            String url = data.getStringExtra("url");
            if (url != null && !url.isEmpty()) {
                mUrlField.setText(url);
                loadUrlFromField();
            }
        }
    }

    @Override
    public void onMediaSessionMetadata(String title, String artist, String album) {
        if (mMediaSession == null) {
            mMediaSession = new MediaSession(mServoView, getApplicationContext());
        }
        Log.d("onMediaSessionMetadata", title + " " + artist + " " + album);
        mMediaSession.updateMetadata(title, artist, album);
    }

    @Override
    public void onMediaSessionPlaybackStateChange(int state) {
        Log.d("onMediaSessionPlaybackStateChange", String.valueOf(state));
        if (mMediaSession == null) {
            mMediaSession = new MediaSession(mServoView, getApplicationContext());
        }

        mMediaSession.setPlaybackState(state);

        if (state == MediaSession.PLAYBACK_STATE_NONE) {
            mMediaSession.hideMediaSessionControls();
            return;
        }
        if (state == MediaSession.PLAYBACK_STATE_PLAYING ||
                state == MediaSession.PLAYBACK_STATE_PAUSED) {
            mMediaSession.showMediaSessionControls();
        }
    }

    @Override
    public void onMediaSessionSetPositionState(float duration, float position, float playbackRate) {
        Log.d("onMediaSessionSetPositionState", duration + " " + position + " " + playbackRate);
    }

    private void onAnimatingIndicatorPrefChanged(boolean value) {
        if (value) {
            mIdleText.setVisibility(View.VISIBLE);
        } else {
            mIdleText.setVisibility(View.GONE);
        }
    }

    private void onExperimentalPrefChanged(boolean value) {
        mServoView.setExperimentalMode(value);
    }

    private void updateSettingsIfNecessary(boolean force) {
        SharedPreferences preferences = PreferenceManager.getDefaultSharedPreferences(getApplicationContext());
        Settings updated = new Settings(preferences);

        if (force || updated.showAnimatingIndicator != mSettings.showAnimatingIndicator) {
            onAnimatingIndicatorPrefChanged(updated.showAnimatingIndicator);
        }

        if (force || updated.experimental != mSettings.experimental) {
            onExperimentalPrefChanged(updated.experimental);
        }

        mSettings = updated;
    }
}
