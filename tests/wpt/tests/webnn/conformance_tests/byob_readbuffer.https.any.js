// META: title=test WebNN API buffer operations
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

// https://www.w3.org/TR/webnn/#api-mlbuffer

const testContents = Uint32Array.from([0, 1, 2, 3, 4, 5, 6, 7]);

let mlContext;
let mlBuffer;
promise_setup(async () => {
  try {
    mlContext = await navigator.ml.createContext(contextOptions);
  } catch (e) {
    throw new AssertionError(
        `Unable to create context for ${variant} variant. ${e}`);
  }

  try {
    mlBuffer = await mlContext.createBuffer({
      dataType: 'int32',
      dimensions: [2, 4],
      usage: MLTensorUsage.WRITE_TO | MLTensorUsage.READ_FROM,
    });
  } catch (e) {
    throw new AssertionError(
        `Unable to create buffer for ${variant} variant. ${e}`);
  }

  mlContext.writeBuffer(mlBuffer, testContents);
});

promise_test(async (t) => {
  const arrayBuffer = new ArrayBuffer(testContents.byteLength - 4);

  await promise_rejects_js(
      t, TypeError, mlContext.readBuffer(mlBuffer, arrayBuffer));
}, `readBuffer() with an ArrayBuffer that is too small should reject`);

promise_test(async (t) => {
  const typedArray = new Uint32Array(testContents.length - 1);

  await promise_rejects_js(
      t, TypeError, mlContext.readBuffer(mlBuffer, typedArray));
}, `readBuffer() with a TypedArray that is too small should reject`);

promise_test(async (t) => {
  const arrayBuffer = new ArrayBuffer(testContents.byteLength);
  const typedArray = new Uint32Array(arrayBuffer);

  arrayBuffer.transfer();

  await promise_rejects_js(
      t, TypeError, mlContext.readBuffer(mlBuffer, arrayBuffer));

  await promise_rejects_js(
      t, TypeError, mlContext.readBuffer(mlBuffer, typedArray));
}, `readBuffer() with a detached ArrayBuffer should reject`);

promise_test(async (t) => {
  const arrayBuffer = new ArrayBuffer(testContents.byteLength);
  const typedArray = new Uint32Array(arrayBuffer);

  const checks = Promise.all([
    promise_rejects_js(
        t, TypeError, mlContext.readBuffer(mlBuffer, arrayBuffer)),
    promise_rejects_js(
        t, TypeError, mlContext.readBuffer(mlBuffer, typedArray)),
  ]);

  arrayBuffer.transfer();

  await checks;
}, `Detaching an ArrayBuffer while readBuffer() is in progress should reject`);

promise_test(async () => {
  const arrayBuffer = new ArrayBuffer(testContents.byteLength);

  await mlContext.readBuffer(mlBuffer, arrayBuffer);

  assert_array_equals(new Uint32Array(arrayBuffer), testContents);
}, `readBuffer() with an ArrayBuffer`);

promise_test(async () => {
  // Create a slightly larger ArrayBuffer and set up the TypedArray at an
  // offset to make sure the MLTensor contents are written to the correct
  // offset.
  const arrayBuffer = new ArrayBuffer(testContents.byteLength + 4);
  const typedArray = new Uint32Array(arrayBuffer, 4);

  await mlContext.readBuffer(mlBuffer, typedArray);

  assert_array_equals(typedArray, testContents);
}, `readBuffer() with a TypedArray`);

promise_test(async () => {
  const arrayBuffer = new ArrayBuffer(testContents.byteLength * 2);

  await mlContext.readBuffer(mlBuffer, arrayBuffer);

  assert_array_equals(
      new Uint32Array(arrayBuffer).subarray(0, testContents.length),
      testContents);
  // The rest of the array should remain uninitialized.
  assert_array_equals(
      new Uint32Array(arrayBuffer)
          .subarray(testContents.length, testContents.length * 2),
      new Uint32Array(testContents.length));
}, `readBuffer() with a larger ArrayBuffer`);

promise_test(async () => {
  // Create a slightly larger ArrayBuffer and set up the TypedArray at an
  // offset to make sure the MLTensor contents are written to the correct
  // offset.
  const arrayBuffer = new ArrayBuffer(testContents.byteLength * 2 + 4);
  const typedArray = new Uint32Array(arrayBuffer, 4);

  await mlContext.readBuffer(mlBuffer, typedArray);

  assert_array_equals(
      typedArray.subarray(0, testContents.length), testContents);
  // The rest of the array should remain uninitialized.
  assert_array_equals(
      typedArray.subarray(testContents.length, testContents.length * 2),
      new Uint32Array(testContents.length));
}, `readBuffer() with a larger TypedArray`);

promise_test(async (t) => {
  const buffer = await mlContext.createBuffer({
    dataType: 'int32',
    dimensions: [2, 2],
    usage: MLTensorUsage.READ_FROM,
  });
  const arrayBufferView = new Int32Array(2 * 2);
  const arrayBuffer = arrayBufferView.buffer;

  // Reading a destroyed MLTensor should reject.
  buffer.destroy();

  await promise_rejects_dom(
      t, 'InvalidStateError', mlContext.readBuffer(buffer, arrayBuffer));
  await promise_rejects_dom(
      t, 'InvalidStateError', mlContext.readBuffer(buffer, arrayBufferView));
}, `readBuffer() rejects on a destroyed MLTensor`);

promise_test(async (t) => {
  const buffer = await mlContext.createBuffer({
    dataType: 'int32',
    dimensions: [2, 2],
    usage: MLTensorUsage.READ_FROM,
  });
  const arrayBufferView = new Int32Array(2 * 2);
  const arrayBuffer = arrayBufferView.buffer;

  const checks = Promise.all([
    promise_rejects_dom(
        t, 'InvalidStateError', mlContext.readBuffer(buffer, arrayBuffer)),
    promise_rejects_dom(
        t, 'InvalidStateError', mlContext.readBuffer(buffer, arrayBufferView)),
  ]);

  buffer.destroy();

  await checks;
}, `readBuffer() rejects when the MLTensor is destroyed`);
