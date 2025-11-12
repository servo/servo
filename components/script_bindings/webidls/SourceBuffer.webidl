/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/*
 * The origin of this IDL file is
 * https://www.w3.org/TR/media-source-2/#sourcebuffer
 *
 */

// https://www.w3.org/TR/media-source-2/#dom-appendmode
enum AppendMode {
  "segments",
  "sequence",
};

// https://www.w3.org/TR/media-source-2/#dom-sourcebuffer
[Pref="dom_media_source_extensions_enabled", Exposed=(Window)]
interface SourceBuffer : EventTarget {
  attribute AppendMode mode;
  readonly  attribute boolean updating;
  readonly  attribute TimeRanges buffered;
  attribute double timestampOffset;
  readonly  attribute AudioTrackList audioTracks;
  readonly  attribute VideoTrackList videoTracks;
  readonly  attribute TextTrackList textTracks;
  attribute double appendWindowStart;
  attribute unrestricted double appendWindowEnd;

  attribute EventHandler onupdatestart;
  attribute EventHandler onupdate;
  attribute EventHandler onupdateend;
  attribute EventHandler onerror;
  attribute EventHandler onabort;

  undefined appendBuffer(BufferSource data);
  undefined abort();
  undefined changeType(DOMString type);
  undefined remove(double start, unrestricted double end);
};
