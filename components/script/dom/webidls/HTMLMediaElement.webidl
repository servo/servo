/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlmediaelement
enum CanPlayTypeResult { "" /* empty string */, "maybe", "probably" };
[Abstract]
interface HTMLMediaElement : HTMLElement {
  // error state
  readonly attribute MediaError? error;

  // network state
  [CEReactions]
           attribute DOMString src;
  readonly attribute DOMString currentSrc;
  // [CEReactions]
  //          attribute DOMString crossOrigin;
  const unsigned short NETWORK_EMPTY = 0;
  const unsigned short NETWORK_IDLE = 1;
  const unsigned short NETWORK_LOADING = 2;
  const unsigned short NETWORK_NO_SOURCE = 3;
  readonly attribute unsigned short networkState;
  [CEReactions]
           attribute DOMString preload;
  // readonly attribute TimeRanges buffered;
  void load();
  CanPlayTypeResult canPlayType(DOMString type);

  // ready state
  const unsigned short HAVE_NOTHING = 0;
  const unsigned short HAVE_METADATA = 1;
  const unsigned short HAVE_CURRENT_DATA = 2;
  const unsigned short HAVE_FUTURE_DATA = 3;
  const unsigned short HAVE_ENOUGH_DATA = 4;
  readonly attribute unsigned short readyState;
  // readonly attribute boolean seeking;

  // playback state
  //          attribute double currentTime;
  // void fastSeek(double time);
  // readonly attribute unrestricted double duration;
  // Date getStartDate();
  readonly attribute boolean paused;
  //          attribute double defaultPlaybackRate;
  //          attribute double playbackRate;
  // readonly attribute TimeRanges played;
  // readonly attribute TimeRanges seekable;
  // readonly attribute boolean ended;
  [CEReactions]
           attribute boolean autoplay;
  // [CEReactions]
  //          attribute boolean loop;
  Promise<void> play();
  void pause();

  // media controller
  //          attribute DOMString mediaGroup;
  //          attribute MediaController? controller;

  // controls
  // [CEReactions]
  //          attribute boolean controls;
  //          attribute double volume;
  //          attribute boolean muted;
  // [CEReactions]
  //          attribute boolean defaultMuted;

  // tracks
  // readonly attribute AudioTrackList audioTracks;
  // readonly attribute VideoTrackList videoTracks;
  // readonly attribute TextTrackList textTracks;
  // TextTrack addTextTrack(TextTrackKind kind, optional DOMString label = "", optional DOMString language = "");
};
