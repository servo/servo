/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

package org.servo.servoshell;

import android.app.Activity;
import android.app.AlertDialog;
import android.content.Context;
import android.content.Intent;
import android.content.SharedPreferences;
import android.net.Uri;
import android.os.Bundle;
import android.preference.PreferenceManager;
import android.system.ErrnoException;
import android.system.Os;
import android.util.Log;
import android.view.KeyEvent;
import android.view.View;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputMethodManager;
import android.widget.Button;
import android.widget.EditText;
import android.widget.ImageButton;
import android.widget.ProgressBar;
import android.widget.TextView;

import org.servo.servoview.Servo;
import org.servo.servoview.ServoView;

import java.io.File;

public class MainActivity extends Activity implements Servo.Client {

    private static final String LOGTAG = "MainActivity";

    ServoView mServoView;
    ImageButton mBackButton;
    ImageButton mFwdButton;
    ImageButton mReloadButton;
    ImageButton mStopButton;
    EditText mUrlField;
    boolean mUrlFieldIsFocused;
    ProgressBar mProgressBar;
    TextView mIdleText;
    boolean mCanGoBack;
    MediaSession mMediaSession;

    class Settings {
        Settings(SharedPreferences preferences) {
            showAnimatingIndicator = preferences.getBoolean("animating_indicator", false);
            experimental = preferences.getBoolean("experimental", false);
        }

        boolean showAnimatingIndicator;
        boolean experimental;
    }
    Settings mSettings;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        mServoView = findViewById(R.id.servoview);
        mBackButton = findViewById(R.id.backbutton);
        mFwdButton = findViewById(R.id.forwardbutton);
        mReloadButton = findViewById(R.id.reloadbutton);
        mStopButton = findViewById(R.id.stopbutton);
        mUrlField = findViewById(R.id.urlfield);
        mUrlFieldIsFocused = false;
        mProgressBar = findViewById(R.id.progressbar);
        mIdleText = findViewById(R.id.redrawing);
        mCanGoBack = false;

        updateSettingsIfNecessary(true);

        mBackButton.setEnabled(false);
        mFwdButton.setEnabled(false);

        // Avoid reload/stop icons doubling up on launch
        mReloadButton.setVisibility(View.GONE);

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

    // From activity_main.xml:
    public void onSettingsClicked(View v) {
        Intent myIntent = new Intent(this, SettingsActivity.class);
        startActivity(myIntent);
    }

    public void onReloadClicked(View v) {
        mServoView.reload();
    }

    public void onBackClicked(View v) {
        mServoView.goBack();
    }

    public void onForwardClicked(View v) {
        mServoView.goForward();
    }

    public void onStopClicked(View v) {
        mServoView.stop();
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
        mReloadButton.setEnabled(false);
        mStopButton.setEnabled(true);
        mReloadButton.setVisibility(View.GONE);
        mStopButton.setVisibility(View.VISIBLE);
        mProgressBar.setVisibility(View.VISIBLE);
    }

    @Override
    public void onLoadEnded() {
        mReloadButton.setEnabled(true);
        mStopButton.setEnabled(false);
        mReloadButton.setVisibility(View.VISIBLE);
        mStopButton.setVisibility(View.GONE);
        mProgressBar.setVisibility(View.INVISIBLE);
    }

    @Override
    public void onTitleChanged(String title) {
    }

    @Override
    public void onUrlChanged(String url) {
        mUrlField.setText(url);
    }

    @Override
    public void onHistoryChanged(boolean canGoBack, boolean canGoForward) {
        mBackButton.setEnabled(canGoBack);
        mFwdButton.setEnabled(canGoForward);
        mCanGoBack = canGoBack;
    }

    public void onRedrawing(boolean redrawing) {
        if (redrawing) {
            mIdleText.setText("LOOP");
        } else {
            mIdleText.setText("IDLE");
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
    public void onMediaSessionMetadata(String title, String artist, String album) {
        if (mMediaSession == null) {
            mMediaSession = new MediaSession(mServoView, this, getApplicationContext());
        }
        Log.d("onMediaSessionMetadata", title + " " + artist + " " + album);
        mMediaSession.updateMetadata(title, artist, album);
    }

    @Override
    public void onMediaSessionPlaybackStateChange(int state) {
        Log.d("onMediaSessionPlaybackStateChange", String.valueOf(state));
        if (mMediaSession == null) {
            mMediaSession = new MediaSession(mServoView, this, getApplicationContext());
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
        if (mMediaSession == null) {
            mMediaSession = new MediaSession(mServoView, this, getApplicationContext());
        }

        mMediaSession.setPositionState(duration, position, playbackRate);
    }

    public void onAnimatingIndicatorPrefChanged(boolean value) {
        if (value) {
            mIdleText.setVisibility(View.VISIBLE);
        } else {
            mIdleText.setVisibility(View.GONE);
        }
    }

    public void onExperimentalPrefChanged(boolean value) {
        mServoView.setExperimentalMode(value);
    }

    public void updateSettingsIfNecessary(boolean force) {
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
