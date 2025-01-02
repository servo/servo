// META: title=test WebNN API constant with shared array buffer
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// Skip tests if WebNN is unimplemented.
promise_setup(async () => {
  assert_implements(navigator.ml, 'missing navigator.ml');
});

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-constant-buffer

const testContents = Int32Array.from([0, 1, 2, 3, 4, 5, 6, 7]);
const sharedArrayBuffer = new SharedArrayBuffer(testContents.byteLength);
const typedArray = new Int32Array(sharedArrayBuffer);
typedArray.set(testContents);

let mlContext;
let mlGraph;
let outputTensor;
promise_setup(async () => {
  try {
    mlContext = await navigator.ml.createContext(contextOptions);
  } catch (e) {
    throw new AssertionError(
        `Unable to create mlContext for ${variant} variant. ${e}`);
  }

  try {
    outputTensor = await mlContext.createTensor({
      dataType: 'int32',
      shape: [8],
      readable: true,
    });
  } catch (e) {
    throw new AssertionError(
        `Unable to create tensor for ${variant} variant. ${e}`);
  }
});

promise_test(async () => {
  const builder = new MLGraphBuilder(mlContext);
  const constant =
      builder.constant({dataType: 'int32', shape: [8]}, sharedArrayBuffer);
  const output = builder.identity(constant);
  const mlGraph = await builder.build({output});

  mlContext.dispatch(mlGraph, {}, {output: outputTensor});
  const results = new Int32Array(await mlContext.readTensor(outputTensor));

  assert_array_equals(results, testContents);
}, `constant() with a SharedArrayBuffer`);

promise_test(async () => {
  const builder = new MLGraphBuilder(mlContext);
  const constant =
      builder.constant({dataType: 'int32', shape: [8]}, typedArray);
  const output = builder.identity(constant);
  const mlGraph = await builder.build({output});

  mlContext.dispatch(mlGraph, {}, {output: outputTensor});
  const results = new Int32Array(await mlContext.readTensor(outputTensor));

  assert_array_equals(results, testContents);
}, `constant() with a typeArray from a SharedArrayBuffer`);
