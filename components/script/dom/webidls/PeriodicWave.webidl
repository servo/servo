/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#periodicwave
 */

dictionary PeriodicWaveConstraints {
  boolean disableNormalization = false;
};

dictionary PeriodicWaveOptions : PeriodicWaveConstraints {
  sequence<float> real;
  sequence<float> imag;
};

[Exposed=Window,
 Constructor(BaseAudioContext context, optional PeriodicWaveOptions options)]
interface PeriodicWave {
};
