/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
package org.servo.servoshell

import android.app.Notification
import android.app.Notification.MediaStyle
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.graphics.drawable.Icon
import android.util.Log
import androidx.core.content.ContextCompat
import androidx.core.content.getSystemService
import org.servo.servoview.ServoView

class MediaSession(private val view: ServoView, private val context: Context) {
    private class NotificationID {
        private var lastID = 0
        val next: Int
            get() {
                lastID++
                return lastID
            }

        fun get(): Int {
            return lastID
        }
    }

    private val notificationID = NotificationID()
    private var mediaSessionActionReceiver: BroadcastReceiver? = null

    private var playbackState = PLAYBACK_STATE_PAUSED

    private var title: String? = null
    private var artist: String? = null
    private var album: String? = null

    init {
        createMediaNotificationChannel()
    }

    private fun createMediaNotificationChannel() {
        val channel = NotificationChannel(
            MEDIA_CHANNEL_ID,
            context.getString(R.string.media_channel_name),
            NotificationManager.IMPORTANCE_LOW,
        )
        channel.description = context.getString(R.string.media_channel_description)
        val notificationManager = context.getSystemService<NotificationManager>()!!
        notificationManager.createNotificationChannel(channel)
    }

    fun showMediaSessionControls() {
        Log.d("MediaSession", "showMediaSessionControls $playbackState")
        val filter = IntentFilter()
        if (playbackState == PLAYBACK_STATE_PAUSED) {
            filter.addAction(KEY_MEDIA_PLAY)
        }
        if (playbackState == PLAYBACK_STATE_PLAYING) {
            filter.addAction(KEY_MEDIA_PAUSE)
        }

        val id: Int
        if (mediaSessionActionReceiver == null) {
            id = notificationID.next

            mediaSessionActionReceiver = object : BroadcastReceiver() {
                override fun onReceive(context: Context?, intent: Intent) {
                    if (intent.action == KEY_MEDIA_PAUSE) {
                        view.mediaSessionAction(ACTION_PAUSE)
                        Log.d("MediaSession", "PAUSE action")
                    } else if (intent.action == KEY_MEDIA_PLAY) {
                        view.mediaSessionAction(ACTION_PLAY)
                        Log.d("MediaSession", "PLAY action")
                    }
                }
            }
        } else {
            id = notificationID.get()
        }

        ContextCompat.registerReceiver(context, mediaSessionActionReceiver, filter, ContextCompat.RECEIVER_NOT_EXPORTED)

        val builder = Notification.Builder(context, MEDIA_CHANNEL_ID)
            .setSmallIcon(R.drawable.media_session_icon)
            .setContentTitle(title)
            .setVisibility(Notification.VISIBILITY_PUBLIC)

        var contentText = ""
        if (!artist.isNullOrEmpty()) {
            contentText = artist!!
        }
        if (!album.isNullOrEmpty()) {
            if (contentText.isNotEmpty()) {
                contentText += " - $album"
            } else {
                contentText = album!!
            }
        }

        if (contentText.isNotEmpty()) {
            builder.setContentText(contentText)
        }

        if (playbackState == PLAYBACK_STATE_PAUSED) {
            builder.addAction(
                Notification.Action.Builder(
                    Icon.createWithResource(context, R.drawable.media_session_play),
                    "Play",
                    PendingIntent.getBroadcast(context, 0, Intent(KEY_MEDIA_PLAY), PendingIntent.FLAG_IMMUTABLE),
                ).build()
            )
        }

        if (playbackState == PLAYBACK_STATE_PLAYING) {
            builder.addAction(
                Notification.Action.Builder(
                    Icon.createWithResource(context, R.drawable.media_session_pause),
                    "Pause",
                    PendingIntent.getBroadcast(context, 0, Intent(KEY_MEDIA_PAUSE), PendingIntent.FLAG_IMMUTABLE),
                ).build()
            )
        }

        builder.setStyle(MediaStyle().setShowActionsInCompactView(0))

        val notificationManager = context.getSystemService<NotificationManager>()!!
        notificationManager.notify(id, builder.build())
    }

    fun hideMediaSessionControls() {
        Log.d("MediaSession", "hideMediaSessionControls")
        val notificationManager = context.getSystemService<NotificationManager>()!!
        notificationManager.cancel(notificationID.get())
        context.unregisterReceiver(mediaSessionActionReceiver)
        mediaSessionActionReceiver = null
    }

    fun setPlaybackState(state: Int) {
        playbackState = state
    }

    fun updateMetadata(title: String?, artist: String?, album: String?) {
        this.title = title
        this.artist = artist
        this.album = album

        if (mediaSessionActionReceiver != null) {
            showMediaSessionControls()
        }
    }

    companion object {
        // https://w3c.github.io/mediasession/#enumdef-mediasessionplaybackstate
        const val PLAYBACK_STATE_NONE: Int = 1
        const val PLAYBACK_STATE_PLAYING: Int = 2
        const val PLAYBACK_STATE_PAUSED: Int = 3

        // https://w3c.github.io/mediasession/#enumdef-mediasessionaction
        private const val ACTION_PLAY = 1
        private const val ACTION_PAUSE = 2
        private const val ACTON_SEEK_BACKWARD = 3
        private const val ACTION_SEEK_FORWARD = 4
        private const val ACTION_PREVIOUS_TRACK = 5
        private const val ACTION_NEXT_TRACK = 6
        private const val ACTION_SKIP_AD = 7
        private const val ACTION_STOP = 8
        private const val ACTION_SEEK_TO = 9

        private const val MEDIA_CHANNEL_ID = "MediaNotificationChannel"
        private const val KEY_MEDIA_PLAY = "org.servo.servoview.MainActivity.play"
        private const val KEY_MEDIA_PAUSE = "org.servo.servoview.MainActivity.pause"
        private const val KEY_MEDIA_PREV = "org.servo.servoview.MainActivity.prev"
        private const val KEY_MEDIA_NEXT = "org.servo.servoview.MainActivity.next"
        private const val KEY_MEDIA_STOP = "org.servo.servoview.MainActivity.stop"
    }
}
