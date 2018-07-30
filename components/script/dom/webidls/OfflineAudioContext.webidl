/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#OfflineAudioContext
 */

dictionary OfflineAudioContextOptions {
  unsigned long numberOfChannels = 1;
  required unsigned long length;
  required float sampleRate;
};

[Exposed=Window,
 Constructor (OfflineAudioContextOptions contextOptions),
 Constructor (unsigned long numberOfChannels, unsigned long length, float sampleRate)]
interface OfflineAudioContext : BaseAudioContext {
  readonly attribute unsigned long length;
  attribute EventHandler oncomplete;

  Promise<AudioBuffer> startRendering();
//  Promise<void> suspend(double suspendTime);
};
