/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#gainnode
 */

dictionary GainOptions : AudioNodeOptions {
  float gain = 1.0;
};

[Exposed=Window]
 interface GainNode : AudioNode {
   [Throws] constructor(BaseAudioContext context, optional GainOptions options = {});
   readonly attribute AudioParam gain;
 };
