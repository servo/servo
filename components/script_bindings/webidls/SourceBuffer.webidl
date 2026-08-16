/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/media-source/#idl-def-sourcebuffer

enum AppendMode {
  "segments",
  "sequence",
};

[Exposed=Window, Pref="dom_media_source_enabled"]
interface SourceBuffer : EventTarget {
  [SetterThrows] attribute AppendMode mode;
  readonly attribute boolean updating;
  [Throws] readonly attribute TimeRanges buffered;
  [SetterThrows] attribute double timestampOffset;
  readonly attribute AudioTrackList audioTracks;
  readonly attribute VideoTrackList videoTracks;
  readonly attribute TextTrackList textTracks;
  [SetterThrows] attribute double appendWindowStart;
  [SetterThrows] attribute unrestricted double appendWindowEnd;

  attribute EventHandler onupdatestart;
  attribute EventHandler onupdate;
  attribute EventHandler onupdateend;
  attribute EventHandler onerror;
  attribute EventHandler onabort;

  [Throws] undefined appendBuffer(BufferSource data);
  [Throws] undefined abort();
  [Throws] undefined changeType(DOMString type);
  [Throws] undefined remove(double start, unrestricted double end);
};
