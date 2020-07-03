/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#mediastreamaudiosourcenode
 */

dictionary MediaStreamAudioSourceOptions {
  required MediaStream mediaStream;
};

[Exposed=Window]
interface MediaStreamAudioSourceNode : AudioNode {
  [Throws] constructor (AudioContext context, MediaStreamAudioSourceOptions options);
  [SameObject] readonly attribute MediaStream mediaStream;
};
