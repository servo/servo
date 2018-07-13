/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#AudioBufferSourceNode
 */

dictionary AudioBufferSourceOptions {
//  AudioBuffer? buffer;
  float detune = 0;
  boolean loop = false;
  double loopEnd = 0;
  double loopStart = 0;
  float playbackRate = 1;
};

[Exposed=Window,
 Constructor (BaseAudioContext context, optional AudioBufferSourceOptions options)]
interface AudioBufferSourceNode : AudioScheduledSourceNode {
  // attribute AudioBuffer? buffer;
  readonly attribute AudioParam playbackRate;
  readonly attribute AudioParam detune;
  attribute boolean loop;
  attribute double loopStart;
  attribute double loopEnd;
  void start(optional double when = 0,
             optional double offset,
             optional double duration);
};
