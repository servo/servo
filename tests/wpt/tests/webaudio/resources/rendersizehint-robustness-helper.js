// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

const ROBUSTNESS_TEST_CONFIGS = [
  {sampleRate: 3000, renderSizeHint: 1, length: 512},
  {sampleRate: 48000, renderSizeHint: 256, length: 1024},
  {sampleRate: 48000, renderSizeHint: 100, length: 1024},
  {sampleRate: 192000, renderSizeHint: 4096, length: 8192},
  {sampleRate: 11000, renderSizeHint: 66000, length: 132000}
];

/**
 * Runs a robustness/stability test for a given node processor.
 *
 * @param {object} config The test configuration containing sampleRate,
 *     renderSizeHint, and length.
 * @param {function(OfflineAudioContext): AudioNode} createTestSetupFunc A
 *     factory function to create and setup the processor node under test.
 * @param {string} testName The name of the test case.
 * @param {function(OfflineAudioContext): void} [postRenderAssertFunc] An
 *     optional hook to run assertions after rendering completes.
 */
const runQuantumRobustnessTest = (
    config, createTestSetupFunc, testName, postRenderAssertFunc) => {
  const {sampleRate, renderSizeHint, length} = config;

  promise_test(async (t) => {
    const audioContext = new OfflineAudioContext({
      numberOfChannels: 1,
      length,
      sampleRate,
      renderSizeHint
    });

    const source = new ConstantSourceNode(audioContext);
    const node = createTestSetupFunc(audioContext, t);

    source.connect(node).connect(audioContext.destination);
    source.start();

    await audioContext.startRendering();

    if (postRenderAssertFunc) {
      postRenderAssertFunc(audioContext, t);
    }
  }, testName);
};
