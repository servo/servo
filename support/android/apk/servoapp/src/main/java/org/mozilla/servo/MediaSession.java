/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

package org.mozilla.servo;

import android.app.Activity;
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

import org.mozilla.servoview.ServoView;

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

    private static final String MEDIA_CHANNEL_ID = "MediaNotificationChannel";
    private static final String KEY_MEDIA_PAUSE = "org.mozilla.servoview.MainActivity.pause";
    private static final String KEY_MEDIA_PREV = "org.mozilla.servoview.MainActivity.prev";
    private static final String KEY_MEDIA_NEXT = "org.mozilla.servoview.MainActivity.next";
    private static final String KEY_MEDIA_STOP = "org.mozilla.servoview.MainActivity.stop";

    ServoView mView;
    MainActivity mActivity;
    Context mContext;
    NotificationID mNotificationID;
    BroadcastReceiver mMediaSessionActionReceiver;

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
          int importance = NotificationManager.IMPORTANCE_DEFAULT;
          NotificationChannel channel =
            new NotificationChannel(MEDIA_CHANNEL_ID, name, importance);
          channel.setDescription(description);
          NotificationManager notificationManager =
            mContext.getSystemService(NotificationManager.class);
          notificationManager.createNotificationChannel(channel);
      }
    }

    public void showMediaSessionControls() {
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

      mContext.registerReceiver(mMediaSessionActionReceiver, filter);

      Intent pauseIntent = new Intent(KEY_MEDIA_PAUSE);
      Notification.Action pauseAction =
        new Notification.Action(R.drawable.media_session_pause, "Pause",
          PendingIntent.getBroadcast(mContext, 0, pauseIntent, 0));

      Intent nextIntent = new Intent(KEY_MEDIA_NEXT);
      Notification.Action nextAction =
        new Notification.Action(R.drawable.media_session_next, "Next",
          PendingIntent.getBroadcast(mContext, 0, nextIntent, 0));

      Intent prevIntent = new Intent(KEY_MEDIA_PREV);
      Notification.Action prevAction =
        new Notification.Action(R.drawable.media_session_prev, "Previous",
          PendingIntent.getBroadcast(mContext, 0, prevIntent, 0));

      Intent stopIntent = new Intent(KEY_MEDIA_STOP);
      Notification.Action stopAction =
        new Notification.Action(R.drawable.media_session_stop, "Stop",
          PendingIntent.getBroadcast(mContext, 0, stopIntent, 0));

      Notification.Builder builder = new Notification.Builder(mContext, this.MEDIA_CHANNEL_ID);
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

      NotificationManager notificationManager =
        mContext.getSystemService(NotificationManager.class);
      notificationManager.notify(mNotificationID.getNext(), builder.build());
    }

    public void hideMediaSessionControls() {
      Log.d("SERVOMEDIA", "hideMediaSessionControls");
      NotificationManager notificationManager =
        mContext.getSystemService(NotificationManager.class);
      notificationManager.cancel(mNotificationID.get());
      mContext.unregisterReceiver(mMediaSessionActionReceiver);
    }
}
