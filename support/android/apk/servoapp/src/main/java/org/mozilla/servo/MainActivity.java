/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

package org.mozilla.servo;

import android.app.Activity;
import android.app.AlertDialog;
import android.app.PendingIntent;
import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.system.ErrnoException;
import android.system.Os;
import android.view.View;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputMethodManager;
import android.webkit.URLUtil;
import android.widget.Button;
import android.widget.EditText;
import android.widget.ProgressBar;
import android.widget.TextView;
import android.util.Log;

import org.mozilla.servoview.ServoView;
import org.mozilla.servoview.Servo;

import java.io.File;

public class MainActivity extends Activity implements Servo.Client {

    private class NotificationID {
        private int lastID = 0;
        public int getNext() {
          lastID++;
          return lastID;
        }

        public int get() {
          return lastID;
        }
    }

    private static final String LOGTAG = "MainActivity";
    private static final String MEDIA_CHANNEL_ID = "MediaNotificationChannel";
    private static final String KEY_MEDIA_PAUSE = "org.mozilla.servoview.MainActivity.pause";
    private static final String KEY_MEDIA_PREV = "org.mozilla.servoview.MainActivity.prev";
    private static final String KEY_MEDIA_NEXT = "org.mozilla.servoview.MainActivity.next";
    private static final String KEY_MEDIA_STOP = "org.mozilla.servoview.MainActivity.stop";

    ServoView mServoView;
    Button mBackButton;
    Button mFwdButton;
    Button mReloadButton;
    Button mStopButton;
    EditText mUrlField;
    ProgressBar mProgressBar;
    TextView mIdleText;
    boolean mCanGoBack;
    NotificationID mNotificationID;
    BroadcastReceiver mMediaSessionActionReceiver;

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

        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.O) {
          File sdcard = getExternalFilesDir("");
          String host = sdcard.toPath().resolve("android_hosts").toString();
          try {
            Os.setenv("HOST_FILE", host, false);
          } catch (ErrnoException e) {
            e.printStackTrace();
          }
        }

        Intent intent = getIntent();
        String args = intent.getStringExtra("servoargs");
        String log = intent.getStringExtra("servolog");
        String gstdebug = intent.getStringExtra("gstdebug");
        mServoView.setServoArgs(args, log, gstdebug);

        if (Intent.ACTION_VIEW.equals(intent.getAction())) {
          mServoView.loadUri(intent.getData());
        }
        setupUrlField();

        mNotificationID = new NotificationID();
        createMediaNotificationChannel();
    }

    @Override
    protected void onDestroy() {
      Log.d("SERVOMEDIA", "onDestroy");
      super.onDestroy();
      hideMediaSessionControls();
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

            if (uri.startsWith("http://") && !text.startsWith("http://")) {
                uri = uri.replaceFirst("http://", "https://");
            }
        }

        mServoView.loadUri(Uri.parse(uri));
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
      Log.d("SERVOMEDIA", "METADATA");
    }

    @Override
    public void onMediaSessionPlaybackStateChange(int state) {
      Log.d("SERVOMEDIA", "PLAYBACK STATE CHANGED " + state);
      if (state == 1 /* none */) {
          hideMediaSessionControls();
          return;
      }
      if (state == 2 /* playing */) {
          showMediaSessionControls();
          return;
      }
    }

    private void createMediaNotificationChannel() {
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
          CharSequence name = getString(R.string.media_channel_name);
          String description = getString(R.string.media_channel_description);
          int importance = NotificationManager.IMPORTANCE_DEFAULT;
          NotificationChannel channel = new NotificationChannel(MEDIA_CHANNEL_ID, name, importance);
          channel.setDescription(description);
          NotificationManager notificationManager = getSystemService(NotificationManager.class);
          notificationManager.createNotificationChannel(channel);
      }
    }

    private void showMediaSessionControls() {
      Context context = getApplicationContext();

      IntentFilter filter = new IntentFilter();
      filter.addAction(KEY_MEDIA_PAUSE);
      filter.addAction(KEY_MEDIA_STOP);
      filter.addAction(KEY_MEDIA_PREV);
      filter.addAction(KEY_MEDIA_NEXT);

      mMediaSessionActionReceiver = new BroadcastReceiver() {
        @Override
        public void onReceive(Context context, Intent intent) {
          if (intent.getAction().equals(KEY_MEDIA_PAUSE)) {
            Log.d("SERVOMEDIA", "PAUSE");
          } else if (intent.getAction().equals(KEY_MEDIA_STOP)) {
            Log.d("SERVOMEDIA", "STOP");
          } else if (intent.getAction().equals(KEY_MEDIA_NEXT)) {
            Log.d("SERVOMEDIA", "NEXT");
          } else if (intent.getAction().equals(KEY_MEDIA_PREV)) {
            Log.d("SERVOMEDIA", "PREV");
          }
        }
      };

      registerReceiver(mMediaSessionActionReceiver, filter);

      Intent pauseIntent = new Intent(KEY_MEDIA_PAUSE);
      Notification.Action pauseAction =
        new Notification.Action(R.drawable.media_session_pause, "Pause",
          PendingIntent.getBroadcast(context, 0, pauseIntent, 0));

      Intent nextIntent = new Intent(KEY_MEDIA_NEXT);
      Notification.Action nextAction =
        new Notification.Action(R.drawable.media_session_next, "Next",
          PendingIntent.getBroadcast(context, 0, nextIntent, 0));

      Intent prevIntent = new Intent(KEY_MEDIA_PREV);
      Notification.Action prevAction =
        new Notification.Action(R.drawable.media_session_prev, "Previous",
          PendingIntent.getBroadcast(context, 0, prevIntent, 0));

      Intent stopIntent = new Intent(KEY_MEDIA_STOP);
      Notification.Action stopAction =
        new Notification.Action(R.drawable.media_session_stop, "Stop",
          PendingIntent.getBroadcast(context, 0, stopIntent, 0));

      Notification.Builder builder = new Notification.Builder(context, this.MEDIA_CHANNEL_ID);
      builder
        .setSmallIcon(R.drawable.media_session_icon)
        .setContentTitle("This is the notification title")
        .setVisibility(Notification.VISIBILITY_PUBLIC)
        .addAction(prevAction)
        .addAction(pauseAction)
        .addAction(nextAction)
        .addAction(stopAction)
        .setStyle(new Notification.MediaStyle()
            .setShowActionsInCompactView(1 /* pause button */ )
            .setShowActionsInCompactView(3 /* stop button */));

      NotificationManager notificationManager = getSystemService(NotificationManager.class);
      notificationManager.notify(mNotificationID.getNext(), builder.build());
    }

    private void hideMediaSessionControls() {
      Log.d("SERVOMEDIA", "hideMediaSessionControls");
      NotificationManager notificationManager = getSystemService(NotificationManager.class);
      notificationManager.cancel(mNotificationID.get());
      unregisterReceiver(mMediaSessionActionReceiver);
    }


}
