// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

/**
 * Set up and perform a test with given test options.
 *
 * @param {!object} testOptions
 * @param {string} testOptions.description test description
 * @param {function} testOptions.graphBuilder a test function returns an
 *   OfflineAudioContext.
 * @param {object} testOptions.tracingOptions test options
 * @param {string} testOptions.tracingOptions.targetCategory
 *   target trace category
 * @param {Array<string>} testOptions.tracingOptions.targetEvents
 *   target trace events
 */
function RunWebAudioPerfTest(testOptions) {
  let isDone = false;
  let startTime;

  async function runTest_internal() {
    const context = await testOptions.graphBuilder();
    PerfTestRunner.addRunTestStartMarker();
    startTime = PerfTestRunner.now();
    await context.startRendering();
    PerfTestRunner.measureValueAsync(PerfTestRunner.now() - startTime);
    PerfTestRunner.addRunTestEndMarker();
    if (!isDone)
      runTest_internal();
  }

  PerfTestRunner.startMeasureValuesAsync({
    unit: 'ms',
    description: testOptions.description,
    done: () => isDone = true,
    run: runTest_internal,
    warmUpCount: 2,
    iterationCount: 5,
    tracingCategories: testOptions.tracingOptions.targetCategory,
    traceEventsToMeasure: testOptions.tracingOptions.targetEvents,
  });
}

/**
 * Creates multiple AudioNodes that are serially connected
 *
 * @param {BaseAudioContext} context
 * @param {string} nodeName AudioNode name in string
 * @param {number} numberOfNodes
 * @param {AudioNodeOptions} nodeOptions
 * @returns {object}
 */
function createAndConnectNodesInSeries(
    context, nodeName, numberOfNodes, nodeOptions) {
  const testNodes = [];
  nodeOptions = nodeOptions || {};
  for (let i = 0; i < numberOfNodes; ++i) {
    testNodes.push(new window[nodeName](context, nodeOptions));
    if (i === 0) continue;
    testNodes[i - 1].connect(testNodes[i]);
  }
  return {
    head: testNodes[0],
    tail: testNodes[numberOfNodes - 1],
    nodes: testNodes,
  };
}

/**
 * Creates multiple AudioNodes.
 *
 * @param {BaseAudioContext} context
 * @param {string} nodeName AudioNode name in string
 * @param {number} numberOfNodes
 * @param {AudioNodeOptions} nodeOptions
 * @returns {Array<AudioNode>}
 */
function createNodes(context, nodeName, numberOfNodes, nodeOptions) {
  const testNodes = [];
  nodeOptions = nodeOptions || {};
  for (let i = 0; i < numberOfNodes; ++i)
    testNodes.push(new window[nodeName](context, nodeOptions));
  return testNodes;
}

/**
 * Creates an AudioBuffer with up-ramp samples.
 *
 * @param {number} length number of samples
 * @param {number} sampleRate sample rate
 * @returns {AudioBuffer}
 */
function createMonoRampBuffer(length, sampleRate) {
  let buffer = new AudioBuffer({numberOfChannel: 1, length, sampleRate});
  let channelData = buffer.getChannelData(0);
  for (let i = 0; i < length; ++i)
    channelData[i] = i / length;
  return buffer;
}
