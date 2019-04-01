/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#ConstantSourceNode
 */

dictionary ConstantSourceOptions: AudioNodeOptions {
  float offset = 1;
};

[Exposed=Window,
 Constructor (BaseAudioContext context, optional ConstantSourceOptions options)]
interface ConstantSourceNode : AudioScheduledSourceNode {
  readonly attribute AudioParam offset;
};
