/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#dom-audionode
 */

enum ChannelCountMode {
  "max",
  "clamped-max",
  "explicit"
};

enum ChannelInterpretation {
  "speakers",
  "discrete"
};

dictionary AudioNodeOptions {
   unsigned long         channelCount;
   ChannelCountMode      channelCountMode;
   ChannelInterpretation channelInterpretation;
};

[Exposed=Window]
interface AudioNode : EventTarget {
  [Throws]
  AudioNode connect(AudioNode destinationNode,
                    optional unsigned long output = 0,
                    optional unsigned long input = 0);
  [Throws]
  undefined connect(AudioParam destinationParam,
               optional unsigned long output = 0);
  [Throws]
  undefined disconnect();
  [Throws]
  undefined disconnect(unsigned long output);
  [Throws]
  undefined disconnect(AudioNode destination);
  [Throws]
  undefined disconnect(AudioNode destination, unsigned long output);
  [Throws]
  undefined disconnect(AudioNode destination,
                  unsigned long output,
                  unsigned long input);
  [Throws]
  undefined disconnect(AudioParam destination);
  [Throws]
  undefined disconnect(AudioParam destination, unsigned long output);

  readonly attribute BaseAudioContext context;
  readonly attribute unsigned long numberOfInputs;
  readonly attribute unsigned long numberOfOutputs;

  [SetterThrows]
  attribute unsigned long channelCount;
  [SetterThrows]
  attribute ChannelCountMode channelCountMode;
  [SetterThrows]
  attribute ChannelInterpretation channelInterpretation;
};
