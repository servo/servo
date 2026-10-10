/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#ConvolverNode
 */

dictionary ConvolverOptions : AudioNodeOptions {
    AudioBuffer? buffer;
    boolean disableNormalization = false;
};

[Exposed=Window]
interface ConvolverNode : AudioNode {
    [Throws] constructor (BaseAudioContext context, optional ConvolverOptions options = {});
    [Throws] attribute AudioBuffer? buffer;
    attribute boolean normalize;
};
