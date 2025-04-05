/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#mediastreamaudiodestinationnode
 */

[Exposed=Window]
interface MediaStreamAudioDestinationNode : AudioNode {
  [Throws] constructor (AudioContext context, optional AudioNodeOptions options = {});
  readonly attribute MediaStream stream;
};
