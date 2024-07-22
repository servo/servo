/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

package org.servo.servoshell;

import android.app.Activity;
import android.app.AlertDialog;
import android.content.Context;
import android.content.Intent;
import android.net.Uri;
import android.os.Bundle;
import android.system.ErrnoException;
import android.system.Os;
import android.util.Log;
import android.view.View;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputMethodManager;
import android.widget.Button;
import android.widget.EditText;
import android.widget.ProgressBar;
import android.widget.TextView;

import org.servo.servoview.Servo;
import org.servo.servoview.ServoView;

import java.io.File;

public class MainActivity extends Activity implements Servo.Client {

    private static final String LOGTAG = "MainActivity";

    ServoView mServoView;
    Button mBackButton;
    Button mFwdButton;
    Button mReloadButton;
    Button mStopButton;
    EditText mUrlField;
    ProgressBar mProgressBar;
    TextView mIdleText;
    boolean mCanGoBack;
    MediaSession mMediaSession;

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
        mProgressBar = findViewById(R.id.progressbar);
        mIdleText = findViewById(R.id.redrawing);
        mCanGoBack = false;

        mBackButton.setEnabled(false);
        mFwdButton.setEnabled(false);

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
        mServoView.setServoArgs(args, log);

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
            if (v.getId() == R.id.urlfield && !hasFocus) {
                InputMethodManager imm =
                        (InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE);
                assert imm != null;
                imm.hideSoftInputFromWindow(v.getWindowToken(), 0);
            }
        });
    }

    private void loadUrlFromField() {
        String text = mUrlField.getText().toString();
        text = text.trim();

        mServoView.loadUri(text);
    }

    // From activity_main.xml:
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

    @Override
    public boolean onAllowNavigation(String url) {
        if (url.startsWith("market://")) {
            try {
                startActivity(new Intent(Intent.ACTION_VIEW, Uri.parse(url)));
                return false;
            } catch (Exception e) {
                Log.e("onAllowNavigation", e.toString());
            }
        }
        return true;
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
}
