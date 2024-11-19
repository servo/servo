/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#biquadfilternode
 */

enum BiquadFilterType {
  "lowpass",
  "highpass",
  "bandpass",
  "lowshelf",
  "highshelf",
  "peaking",
  "notch",
  "allpass"
};

dictionary BiquadFilterOptions : AudioNodeOptions {
  BiquadFilterType type = "lowpass";
  float Q = 1;
  float detune = 0;
  float frequency = 350;
  float gain = 0;
};

[Exposed=Window]
interface BiquadFilterNode : AudioNode {
  [Throws] constructor(BaseAudioContext context, optional BiquadFilterOptions options = {});
  attribute BiquadFilterType type;
  readonly attribute AudioParam frequency;
  readonly attribute AudioParam detune;
  readonly attribute AudioParam Q;
  readonly attribute AudioParam gain;
  // the AudioParam model of https://github.com/servo/servo/issues/21659 needs to
  // be implemented before we implement this
  // void getFrequencyResponse (Float32Array frequencyHz, Float32Array magResponse, Float32Array phaseResponse);
};
