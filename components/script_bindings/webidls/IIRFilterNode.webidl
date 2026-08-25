/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#IIRFilterNode
 */

[Exposed=Window]
interface IIRFilterNode : AudioNode {
  [Throws] constructor (BaseAudioContext context, IIRFilterOptions options);
  [Throws] undefined getFrequencyResponse (
    Float32Array frequencyHz,
    Float32Array magResponse,
    Float32Array phaseResponse
  );
};

dictionary IIRFilterOptions : AudioNodeOptions {
  required sequence<double> feedforward;
  required sequence<double> feedback;
};
