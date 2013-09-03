/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-video-element
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// import from http://mxr.mozilla.org/mozilla-central/source/dom/webidl/

interface HTMLVideoElement : HTMLMediaElement {
  [SetterThrows]
           attribute unsigned long width;
  [SetterThrows]
           attribute unsigned long height;
  readonly attribute unsigned long videoWidth;
  readonly attribute unsigned long videoHeight;
  [SetterThrows]
           attribute DOMString poster;
};
/*
partial interface HTMLVideoElement {
  // A count of the number of video frames that have demuxed from the media
  // resource. If we were playing perfectly, we'd be able to paint this many
  // frames.
  readonly attribute unsigned long mozParsedFrames;

  // A count of the number of frames that have been decoded. We may drop
  // frames if the decode is taking too much time.
  readonly attribute unsigned long mozDecodedFrames;

  // A count of the number of frames that have been presented to the rendering
  // pipeline. We may drop frames if they arrive late at the renderer.
  readonly attribute unsigned long mozPresentedFrames;

  // Number of presented frames which were painted on screen.
  readonly attribute unsigned long mozPaintedFrames;

  // Time which the last painted video frame was late by, in seconds.
  readonly attribute double mozFrameDelay;

  // True if the video has an audio track available.
  readonly attribute boolean mozHasAudio;
};

// https://dvcs.w3.org/hg/html-media/raw-file/default/media-source/media-source.html#idl-def-HTMLVideoElement
partial interface HTMLVideoElement {
  [Pref="media.mediasource.enabled", Creator]
  VideoPlaybackQuality getVideoPlaybackQuality();
};
*/
