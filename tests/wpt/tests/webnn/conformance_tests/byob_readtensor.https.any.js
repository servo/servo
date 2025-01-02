// META: title=test WebNN API tensor operations
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

// https://www.w3.org/TR/webnn/#api-mltensor

const testContents = Uint32Array.from([0, 1, 2, 3, 4, 5, 6, 7]);

let mlContext;
let mlTensor;
promise_setup(async () => {
  try {
    mlContext = await navigator.ml.createContext(contextOptions);
  } catch (e) {
    throw new AssertionError(
        `Unable to create context for ${variant} variant. ${e}`);
  }

  try {
    mlTensor = await mlContext.createTensor({
      dataType: 'int32',
      shape: [2, 4],
      readable: true,
      writable: true,
    });
  } catch (e) {
    throw new AssertionError(
        `Unable to create tensor for ${variant} variant. ${e}`);
  }

  mlContext.writeTensor(mlTensor, testContents);
});

promise_test(async (t) => {
  const arrayBuffer = new ArrayBuffer(testContents.byteLength - 4);

  await promise_rejects_js(
      t, TypeError, mlContext.readTensor(mlTensor, arrayBuffer));
}, `readTensor() with an ArrayBuffer that is too small should reject`);

promise_test(async (t) => {
  const typedArray = new Uint32Array(testContents.length - 1);

  await promise_rejects_js(
      t, TypeError, mlContext.readTensor(mlTensor, typedArray));
}, `readTensor() with a TypedArray that is too small should reject`);

promise_test(async (t) => {
  const arrayBuffer = new ArrayBuffer(testContents.byteLength);
  const typedArray = new Uint32Array(arrayBuffer);

  arrayBuffer.transfer();

  await promise_rejects_js(
      t, TypeError, mlContext.readTensor(mlTensor, arrayBuffer));

  await promise_rejects_js(
      t, TypeError, mlContext.readTensor(mlTensor, typedArray));
}, `readTensor() with a detached ArrayBuffer should reject`);

promise_test(async (t) => {
  const arrayBuffer = new ArrayBuffer(testContents.byteLength);
  const typedArray = new Uint32Array(arrayBuffer);

  const checks = Promise.all([
    promise_rejects_js(
        t, TypeError, mlContext.readTensor(mlTensor, arrayBuffer)),
    promise_rejects_js(
        t, TypeError, mlContext.readTensor(mlTensor, typedArray)),
  ]);

  arrayBuffer.transfer();

  await checks;
}, `Detaching an ArrayBuffer while readTensor() is in progress should reject`);

promise_test(async () => {
  const arrayBuffer = new ArrayBuffer(testContents.byteLength);

  await mlContext.readTensor(mlTensor, arrayBuffer);

  assert_array_equals(new Uint32Array(arrayBuffer), testContents);
}, `readTensor() with an ArrayBuffer`);

promise_test(async () => {
  const sharedArrayBuffer = new SharedArrayBuffer(testContents.byteLength);

  await mlContext.readTensor(mlTensor, sharedArrayBuffer);

  assert_array_equals(new Uint32Array(sharedArrayBuffer), testContents);
}, `readTensor() with a SharedArrayBuffer`);

promise_test(async () => {
  const sharedArrayBuffer = new SharedArrayBuffer(testContents.byteLength);
  const typedArray = new Uint32Array(sharedArrayBuffer);

  await mlContext.readTensor(mlTensor, typedArray);

  assert_array_equals(typedArray, testContents);
}, `readTensor() with a typeArray from a SharedArrayBuffer`);

promise_test(async () => {
  // Create a slightly larger ArrayBuffer and set up the TypedArray at an
  // offset to make sure the MLTensor contents are written to the correct
  // offset.
  const arrayBuffer = new ArrayBuffer(testContents.byteLength + 4);
  const typedArray = new Uint32Array(arrayBuffer, 4);

  await mlContext.readTensor(mlTensor, typedArray);

  assert_array_equals(typedArray, testContents);
}, `readTensor() with a TypedArray`);

promise_test(async () => {
  const arrayBuffer = new ArrayBuffer(testContents.byteLength * 2);

  await mlContext.readTensor(mlTensor, arrayBuffer);

  assert_array_equals(
      new Uint32Array(arrayBuffer).subarray(0, testContents.length),
      testContents);
  // The rest of the array should remain uninitialized.
  assert_array_equals(
      new Uint32Array(arrayBuffer)
          .subarray(testContents.length, testContents.length * 2),
      new Uint32Array(testContents.length));
}, `readTensor() with a larger ArrayBuffer`);

promise_test(async () => {
  // Create a slightly larger ArrayBuffer and set up the TypedArray at an
  // offset to make sure the MLTensor contents are written to the correct
  // offset.
  const arrayBuffer = new ArrayBuffer(testContents.byteLength * 2 + 4);
  const typedArray = new Uint32Array(arrayBuffer, 4);

  await mlContext.readTensor(mlTensor, typedArray);

  assert_array_equals(
      typedArray.subarray(0, testContents.length), testContents);
  // The rest of the array should remain uninitialized.
  assert_array_equals(
      typedArray.subarray(testContents.length, testContents.length * 2),
      new Uint32Array(testContents.length));
}, `readTensor() with a larger TypedArray`);

promise_test(async (t) => {
  const tensor = await mlContext.createTensor({
    dataType: 'int32',
    shape: [2, 2],
    readable: true,
  });
  const arrayBufferView = new Int32Array(2 * 2);
  const arrayBuffer = arrayBufferView.buffer;

  // Reading a destroyed MLTensor should reject.
  tensor.destroy();

  await promise_rejects_dom(
      t, 'InvalidStateError', mlContext.readTensor(tensor, arrayBuffer));
  await promise_rejects_dom(
      t, 'InvalidStateError', mlContext.readTensor(tensor, arrayBufferView));
}, `readTensor() rejects on a destroyed MLTensor`);

promise_test(async (t) => {
  const tensor = await mlContext.createTensor({
    dataType: 'int32',
    shape: [2, 2],
    readable: true,
  });
  const arrayBufferView = new Int32Array(2 * 2);
  const arrayBuffer = arrayBufferView.buffer;

  const checks = Promise.all([
    promise_rejects_dom(
        t, 'InvalidStateError', mlContext.readTensor(tensor, arrayBuffer)),
    promise_rejects_dom(
        t, 'InvalidStateError', mlContext.readTensor(tensor, arrayBufferView)),
  ]);

  tensor.destroy();

  await checks;
}, `readTensor() rejects when the MLTensor is destroyed`);
