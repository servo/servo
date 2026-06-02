/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#mediaelementaudiosourcenode
 */

dictionary MediaElementAudioSourceOptions {
  required HTMLMediaElement mediaElement;
};

[Exposed=Window]
interface MediaElementAudioSourceNode : AudioNode {
  [Throws] constructor (AudioContext context, MediaElementAudioSourceOptions options);
  [SameObject] readonly attribute HTMLMediaElement mediaElement;
};
