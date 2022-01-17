/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#BaseAudioContext
 */

enum AudioContextState {
  "suspended",
  "running",
  "closed"
};

callback DecodeErrorCallback = undefined (DOMException error);
callback DecodeSuccessCallback = undefined (AudioBuffer decodedData);

[Exposed=Window]
interface BaseAudioContext : EventTarget {
  readonly attribute AudioDestinationNode destination;
  readonly attribute float sampleRate;
  readonly attribute double currentTime;
  readonly attribute AudioListener listener;
  readonly attribute AudioContextState  state;
  Promise<undefined> resume();
  attribute EventHandler onstatechange;
  [Throws] AudioBuffer createBuffer(unsigned long numberOfChannels,
                                    unsigned long length,
                                    float sampleRate);
  Promise<AudioBuffer> decodeAudioData(ArrayBuffer audioData,
                                       optional DecodeSuccessCallback successCallback,
                                       optional DecodeErrorCallback errorCallback);
  [Throws] AudioBufferSourceNode createBufferSource();
  [Throws] ConstantSourceNode createConstantSource();
  // ScriptProcessorNode createScriptProcessor(optional unsigned long bufferSize = 0,
  //                                           optional unsigned long numberOfInputChannels = 2,
  //                                           optional unsigned long numberOfOutputChannels = 2);
  [Throws] AnalyserNode createAnalyser();
  [Throws]  GainNode createGain();
  // DelayNode createDelay(optional double maxDelayTime = 1);
  [Throws] BiquadFilterNode createBiquadFilter();
  // IIRFilterNode createIIRFilter(sequence<double> feedforward,
  //                               sequence<double> feedback);
  // WaveShaperNode createWaveShaper();
  [Throws] PannerNode createPanner();
  [Throws] StereoPannerNode createStereoPanner();
  // ConvolverNode createConvolver();
  [Throws] ChannelSplitterNode createChannelSplitter(optional unsigned long numberOfOutputs = 6);
  [Throws] ChannelMergerNode createChannelMerger(optional unsigned long numberOfInputs = 6);
  // DynamicsCompressorNode createDynamicsCompressor();
  [Throws]  OscillatorNode createOscillator();
  // PeriodicWave createPeriodicWave(sequence<float> real,
  //                                 sequence<float> imag,
  //                                 optional PeriodicWaveConstraints constraints);
};
