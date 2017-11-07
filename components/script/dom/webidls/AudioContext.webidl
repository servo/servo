/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
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
  AudioContextLatencyCategory latencyHint = "interactive";
  float sampleRate;
};

dictionary AudioTimestamp {
  double contextTime;
  DOMHighResTimeStamp performanceTime;
};

[Exposed=Window,
 Constructor(optional AudioContextOptions contextOptions)]
interface AudioContext : BaseAudioContext {
  readonly attribute double baseLatency;
  readonly attribute double outputLatency;

  AudioTimestamp getOutputTimestamp();

  Promise<void> suspend();
  Promise<void> close();

  // MediaElementAudioSourceNode createMediaElementSource(HTMLMediaElement mediaElement);
  // MediaStreamAudioSourceNode createMediaStreamSource(MediaStream mediaStream);
  // MediaStreamTrackAudioSourceNode createMediaStreamTrackSource(MediaStreamTrack mediaStreamTrack);
  // MediaStreamAudioDestinationNode createMediaStreamDestination();
};
