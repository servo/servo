/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#PeriodicWave
 */

dictionary DelayOptions : AudioNodeOptions {
    double maxDelayTime = 1;
    double delayTime = 0;
};

[Exposed=Window]
interface DelayNode : AudioNode {
    [Throws] constructor (BaseAudioContext context, optional DelayOptions options = {});
    readonly attribute AudioParam delayTime;
};
