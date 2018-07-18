/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


package com.mozilla.servo;

import android.app.Activity;
import android.content.Context;
import android.content.Intent;
import android.net.Uri;
import android.os.Bundle;
import android.util.Log;
import android.view.View;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputMethodManager;
import android.webkit.URLUtil;
import android.widget.Button;
import android.widget.EditText;
import android.widget.ProgressBar;

import com.mozilla.servoview.ServoView;

public class MainActivity extends Activity implements ServoView.Client {

    private static final String LOGTAG = "MainActivity";

    ServoView mServoView;
    Button mBackButton;
    Button mFwdButton;
    Button mReloadButton;
    Button mStopButton;
    EditText mUrlField;
    ProgressBar mProgressBar;


    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        mServoView = (ServoView)findViewById(R.id.servoview);
        mBackButton = (Button)findViewById(R.id.backbutton);
        mFwdButton = (Button)findViewById(R.id.forwardbutton);
        mReloadButton = (Button)findViewById(R.id.reloadbutton);
        mStopButton = (Button)findViewById(R.id.stopbutton);
        mUrlField = (EditText)findViewById(R.id.urlfield);
        mProgressBar = (ProgressBar)findViewById(R.id.progressbar);

        mServoView.setClient(this);
        mBackButton.setEnabled(false);
        mFwdButton.setEnabled(false);

        mServoView.requestFocus();

        String args = getIntent().getStringExtra("servoargs");
        if (args != null) {
          mServoView.setServoArgs(args);
        }

        setupUrlField();
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
            if(v.getId() == R.id.urlfield && !hasFocus) {
                InputMethodManager imm =  (InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE);
                assert imm != null;
                imm.hideSoftInputFromWindow(v.getWindowToken(), 0);
            }
        });
    }

    private void loadUrlFromField() {
        String text = mUrlField.getText().toString();
        text = text.trim();
        String uri;

        if (text.contains(" ") || !text.contains(".")) {
            uri =  URLUtil.composeSearchUrl(text, "https://duckduckgo.com/html/?q=%s", "%s");
        } else {
            uri = URLUtil.guessUrl(text);
        }

        mServoView.loadUri(Uri.parse(uri));
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
    }

}
