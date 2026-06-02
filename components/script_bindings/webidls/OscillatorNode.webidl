/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#oscillatornode
 */

enum OscillatorType {
  "sine",
  "square",
  "sawtooth",
  "triangle",
  "custom"
};

dictionary OscillatorOptions : AudioNodeOptions {
  OscillatorType type = "sine";
  float frequency = 440;
  float detune = 0;
  // PeriodicWave periodicWave;
};

[Exposed=Window]
interface OscillatorNode : AudioScheduledSourceNode {
  [Throws] constructor(BaseAudioContext context, optional OscillatorOptions options = {});
  [SetterThrows]
  attribute OscillatorType type;

  readonly attribute AudioParam frequency;
  readonly attribute AudioParam detune;

//  void setPeriodicWave (PeriodicWave periodicWave);
};
