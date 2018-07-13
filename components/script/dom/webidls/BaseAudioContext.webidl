/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#BaseAudioContext
 */

enum AudioContextState {
  "suspended",
  "running",
  "closed"
};

// callback DecodeErrorCallback = void (DOMException error);
// callback DecodeSuccessCallback = void (AudioBuffer decodedData);

[Exposed=Window]
interface BaseAudioContext : EventTarget {
  readonly attribute AudioDestinationNode destination;
  readonly attribute float sampleRate;
  readonly attribute double currentTime;
  // readonly attribute AudioListener listener;
  readonly attribute AudioContextState  state;
  Promise<void> resume();
  attribute EventHandler onstatechange;
  AudioBuffer createBuffer(unsigned long numberOfChannels,
                           unsigned long length,
                           float sampleRate);
  // Promise<AudioBuffer> decodeAudioData(ArrayBuffer audioData,
  //                                      optional DecodeSuccessCallback successCallback,
  //                                      optional DecodeErrorCallback errorCallback);
  // AudioBufferSourceNode createBufferSource();
  // ConstantSourceNode createConstantSource();
  // ScriptProcessorNode createScriptProcessor(optional unsigned long bufferSize = 0,
  //                                           optional unsigned long numberOfInputChannels = 2,
  //                                           optional unsigned long numberOfOutputChannels = 2);
  // AnalyserNode createAnalyser();
  GainNode createGain();
  // DelayNode createDelay(optional double maxDelayTime = 1);
  // BiquadFilterNode createBiquadFilter();
  // IIRFilterNode createIIRFilter(sequence<double> feedforward,
  //                               sequence<double> feedback);
  // WaveShaperNode createWaveShaper();
  // PannerNode createPanner();
  // StereoPannerNode createStereoPanner();
  // ConvolverNode createConvolver();
  // ChannelSplitterNode createChannelSplitter(optional unsigned long numberOfOutputs = 6);
  // ChannelMergerNode createChannelMerger(optional unsigned long numberOfInputs = 6);
  // DynamicsCompressorNode createDynamicsCompressor();
  OscillatorNode createOscillator();
  // PeriodicWave createPeriodicWave(sequence<float> real,
  //                                 sequence<float> imag,
  //                                 optional PeriodicWaveConstraints constraints);
};
