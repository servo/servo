/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

package org.servo.servoshell;

import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;
import android.os.Build;
import android.util.Log;

import org.servo.servoview.ServoView;

public class MediaSession {
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

    // https://w3c.github.io/mediasession/#enumdef-mediasessionplaybackstate
    public static final int PLAYBACK_STATE_NONE = 1;
    public static final int PLAYBACK_STATE_PLAYING = 2;
    public static final int PLAYBACK_STATE_PAUSED = 3;

    // https://w3c.github.io/mediasession/#enumdef-mediasessionaction
    private static final int ACTION_PLAY = 1;
    private static final int ACTION_PAUSE = 2;
    private static final int ACTON_SEEK_BACKWARD = 3;
    private static final int ACTION_SEEK_FORWARD = 4;
    private static final int ACTION_PREVIOUS_TRACK = 5;
    private static final int ACTION_NEXT_TRACK = 6;
    private static final int ACTION_SKIP_AD = 7;
    private static final int ACTION_STOP = 8;
    private static final int ACTION_SEEK_TO = 9;

    private static final String MEDIA_CHANNEL_ID = "MediaNotificationChannel";
    private static final String KEY_MEDIA_PLAY = "org.servo.servoview.MainActivity.play";
    private static final String KEY_MEDIA_PAUSE = "org.servo.servoview.MainActivity.pause";
    private static final String KEY_MEDIA_PREV = "org.servo.servoview.MainActivity.prev";
    private static final String KEY_MEDIA_NEXT = "org.servo.servoview.MainActivity.next";
    private static final String KEY_MEDIA_STOP = "org.servo.servoview.MainActivity.stop";

    ServoView mView;
    MainActivity mActivity;
    Context mContext;

    NotificationID mNotificationID;
    BroadcastReceiver mMediaSessionActionReceiver;

    int mPlaybackState = PLAYBACK_STATE_PAUSED;

    String mTitle;
    String mArtist;
    String mAlbum;

    public MediaSession(ServoView view, MainActivity activity, Context context) {
      mView = view;
      mActivity = activity;
      mContext = context;
      mNotificationID = new NotificationID();
      createMediaNotificationChannel();
    }

    private void createMediaNotificationChannel() {
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
          CharSequence name =
            mContext.getResources().getString(R.string.media_channel_name);
          String description =
            mContext.getResources().getString(R.string.media_channel_description);
          int importance = NotificationManager.IMPORTANCE_LOW;
          NotificationChannel channel =
            new NotificationChannel(MEDIA_CHANNEL_ID, name, importance);
          channel.setDescription(description);
          NotificationManager notificationManager =
            mContext.getSystemService(NotificationManager.class);
          notificationManager.createNotificationChannel(channel);
      }
    }

    public void showMediaSessionControls() {
      Log.d("MediaSession", "showMediaSessionControls " + mPlaybackState);
      IntentFilter filter = new IntentFilter();
      if (mPlaybackState == PLAYBACK_STATE_PAUSED) {
        filter.addAction(KEY_MEDIA_PLAY);
      }
      if (mPlaybackState == PLAYBACK_STATE_PLAYING) {
        filter.addAction(KEY_MEDIA_PAUSE);
      }

      int id;
      if (mMediaSessionActionReceiver == null) {
        id = mNotificationID.getNext();

        mMediaSessionActionReceiver = new BroadcastReceiver() {
          @Override
          public void onReceive(Context context, Intent intent) {
            if (intent.getAction().equals(KEY_MEDIA_PAUSE)) {
              mView.mediaSessionAction(ACTION_PAUSE);
              Log.d("MediaSession", "PAUSE action");
            } else if (intent.getAction().equals(KEY_MEDIA_PLAY)) {
              mView.mediaSessionAction(ACTION_PLAY);
              Log.d("MediaSession", "PLAY action");
            }
          }
        };
      } else {
        id = mNotificationID.get();
      }

      mContext.registerReceiver(mMediaSessionActionReceiver, filter);

      Notification.Builder builder = new Notification.Builder(mContext, MEDIA_CHANNEL_ID);
      builder
        .setSmallIcon(R.drawable.media_session_icon)
        .setContentTitle(mTitle)
        .setVisibility(Notification.VISIBILITY_PUBLIC);

      String contentText = new String();
      if (mArtist != null && !mArtist.isEmpty()) {
        contentText = mArtist;
      }
      if (mAlbum != null && !mAlbum.isEmpty()) {
        if (!contentText.isEmpty()) {
          contentText += " - " + mAlbum;
        } else {
          contentText = mAlbum;
        }
      }

      if (!contentText.isEmpty()) {
        builder.setContentText(contentText);
      }

      if (mPlaybackState == PLAYBACK_STATE_PAUSED) {
        Intent playIntent = new Intent(KEY_MEDIA_PLAY);
        Notification.Action playAction =
          new Notification.Action(R.drawable.media_session_play, "Play",
            PendingIntent.getBroadcast(mContext, 0, playIntent, 0));
        builder.addAction(playAction);
      }

      if (mPlaybackState == PLAYBACK_STATE_PLAYING) {
        Intent pauseIntent = new Intent(KEY_MEDIA_PAUSE);
        Notification.Action pauseAction =
          new Notification.Action(R.drawable.media_session_pause, "Pause",
            PendingIntent.getBroadcast(mContext, 0, pauseIntent, 0));
        builder.addAction(pauseAction);
      }

      builder.setStyle(new Notification.MediaStyle()
        .setShowActionsInCompactView(0));

      NotificationManager notificationManager =
        mContext.getSystemService(NotificationManager.class);
      notificationManager.notify(id, builder.build());
    }

    public void hideMediaSessionControls() {
      Log.d("MediaSession", "hideMediaSessionControls");
      NotificationManager notificationManager =
        mContext.getSystemService(NotificationManager.class);
      notificationManager.cancel(mNotificationID.get());
      mContext.unregisterReceiver(mMediaSessionActionReceiver);
      mMediaSessionActionReceiver = null;
    }

    public void setPlaybackState(int state) {
      mPlaybackState = state;
    }

    public void updateMetadata(String title, String artist, String album) {
      mTitle = title;
      mArtist = artist;
      mAlbum = album;

      if (mMediaSessionActionReceiver != null) {
        showMediaSessionControls();
      }
    }

    // Not implemented
    // see https://github.com/servo/servo/pull/24885#discussion_r352496117
    public void setPositionState(float duration, float position, float playbackRate) {}
}
