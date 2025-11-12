/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/*
 * The origin of this IDL file is
 * https://www.w3.org/TR/media-source-2/#mediasource
 *
 */

// https://www.w3.org/TR/media-source-2/#dom-readystate
enum ReadyState {
  "closed",
  "open",
  "ended",
};

// https://www.w3.org/TR/media-source-2/#dom-endofstreamerror
enum EndOfStreamError {
  "network",
  "decode",
};

// https://www.w3.org/TR/media-source-2/#dom-mediasource
[Pref="dom_media_source_extensions_enabled", Exposed=(Window)]
interface MediaSource : EventTarget {
    constructor();

    // [SameObject, Exposed=DedicatedWorker]
    // readonly  attribute MediaSourceHandle handle;
    readonly  attribute SourceBufferList sourceBuffers;
    readonly  attribute SourceBufferList activeSourceBuffers;
    readonly  attribute ReadyState readyState;

    attribute unrestricted double duration;
    attribute EventHandler onsourceopen;
    attribute EventHandler onsourceended;
    attribute EventHandler onsourceclose;

    static readonly attribute boolean canConstructInDedicatedWorker;

    SourceBuffer addSourceBuffer(DOMString type);
    undefined removeSourceBuffer(SourceBuffer sourceBuffer);
    undefined endOfStream(optional EndOfStreamError error);
    undefined setLiveSeekableRange(double start, double end);
    undefined clearLiveSeekableRange();
    static boolean isTypeSupported(DOMString type);
};

