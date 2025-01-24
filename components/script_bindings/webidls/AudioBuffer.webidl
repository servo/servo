/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#audiobuffer
 */

dictionary AudioBufferOptions {
  unsigned long numberOfChannels = 1;
  required unsigned long length;
  required float sampleRate;
};

[Exposed=Window]
interface AudioBuffer {
  [Throws] constructor(AudioBufferOptions options);
  readonly attribute float sampleRate;
  readonly attribute unsigned long length;
  readonly attribute double duration;
  readonly attribute unsigned long numberOfChannels;
  [Throws] Float32Array getChannelData(unsigned long channel);
  [Throws] undefined copyFromChannel(Float32Array destination,
                                unsigned long channelNumber,
                                optional unsigned long startInChannel = 0);
  [Throws] undefined copyToChannel(Float32Array source,
                              unsigned long channelNumber,
                              optional unsigned long startInChannel = 0);
};
