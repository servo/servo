/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://w3c.github.io/mediasession/#mediasession
 */

[Exposed=Window]
partial interface Navigator {
  [SameObject] readonly attribute MediaSession mediaSession;
};

enum MediaSessionPlaybackState {
  "none",
  "paused",
  "playing"
};

enum MediaSessionAction {
  "play",
  "pause",
  "seekbackward",
  "seekforward",
  "previoustrack",
  "nexttrack",
  "skipad",
  "stop",
  "seekto"
};

dictionary MediaSessionActionDetails {
  required MediaSessionAction action;
};

dictionary MediaSessionSeekActionDetails : MediaSessionActionDetails {
  double? seekOffset;
};

dictionary MediaSessionSeekToActionDetails : MediaSessionActionDetails {
  required double seekTime;
  boolean? fastSeek;
};

dictionary MediaPositionState {
  double duration;
  double playbackRate;
  double position;
};

callback MediaSessionActionHandler = undefined(/*MediaSessionActionDetails details*/);

[Exposed=Window]
interface MediaSession {
  attribute MediaMetadata? metadata;

  attribute MediaSessionPlaybackState playbackState;

  undefined setActionHandler(MediaSessionAction action, MediaSessionActionHandler? handler);

  [Throws] undefined setPositionState(optional MediaPositionState state = {});
};
