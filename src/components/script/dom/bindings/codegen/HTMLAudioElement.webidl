/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-audio-element
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// import from http://mxr.mozilla.org/mozilla-central/source/dom/webidl/

[NamedConstructor=Audio(optional DOMString src)]
interface HTMLAudioElement : HTMLMediaElement {};

partial interface HTMLAudioElement
{
/*
  // Setup the audio stream for writing
  [Pref="media.audio_data.enabled", Throws]
  void mozSetup(unsigned long channels, unsigned long rate);

  // Write audio to the audio stream
  [Pref="media.audio_data.enabled", Throws]
  unsigned long mozWriteAudio(Float32Array data);
  [Pref="media.audio_data.enabled", Throws]
  unsigned long mozWriteAudio(sequence<unrestricted float> data);

  // Get the current offset (measured in samples since the start) of the audio
  // stream created using mozWriteAudio().
  [Pref="media.audio_data.enabled", Throws]
  unsigned long long mozCurrentSampleOffset();
*/
};
