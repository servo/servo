/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#dom-audiocontext
 */

enum AudioContextLatencyCategory {
  "balanced",
  "interactive",
  "playback"
};

dictionary AudioContextOptions {
  (AudioContextLatencyCategory or double) latencyHint = "interactive";
  float sampleRate;
};

dictionary AudioTimestamp {
  double contextTime;
  DOMHighResTimeStamp performanceTime;
};

[Exposed=Window]
interface AudioContext : BaseAudioContext {
  [Throws] constructor(optional AudioContextOptions contextOptions = {});
  readonly attribute double baseLatency;
  readonly attribute double outputLatency;

  AudioTimestamp getOutputTimestamp();

  Promise<undefined> suspend();
  Promise<undefined> close();

  [Throws] MediaElementAudioSourceNode createMediaElementSource(HTMLMediaElement mediaElement);
  [Throws] MediaStreamAudioSourceNode createMediaStreamSource(MediaStream mediaStream);
  [Throws] MediaStreamTrackAudioSourceNode createMediaStreamTrackSource(MediaStreamTrack mediaStreamTrack);
  [Throws] MediaStreamAudioDestinationNode createMediaStreamDestination();
};
