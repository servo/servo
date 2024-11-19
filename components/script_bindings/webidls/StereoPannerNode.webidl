/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#StereoPannerNode
 */

dictionary StereoPannerOptions: AudioNodeOptions {
  float pan = 0;
};

[Exposed=Window]
interface StereoPannerNode : AudioScheduledSourceNode {
  [Throws] constructor(BaseAudioContext context, optional StereoPannerOptions options = {});
  readonly attribute AudioParam pan;
};
