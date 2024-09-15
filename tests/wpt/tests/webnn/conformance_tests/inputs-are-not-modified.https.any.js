// META: title=test that input tensors are not modified during a call to dispatch()
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlcontext-dispatch

let mlContext;

// Skip tests if WebNN is unimplemented.
promise_setup(async () => {
  assert_implements(navigator.ml, 'missing navigator.ml');
  mlContext = await navigator.ml.createContext(contextOptions);
});

promise_test(async () => {
  const builder = new MLGraphBuilder(mlContext);
  const inputOperand =
      builder.input('input', {dataType: 'float32', dimensions: [4]});
  const hardSwishOperand = builder.hardSwish(inputOperand);
  // Add some other operator for the output tensor to bind to; otherwise there
  // is no reason to implement hardSwish "in-place".
  const outputOperand = builder.identity(hardSwishOperand);

  const [inputTensor, outputTensor, mlGraph] = await Promise.all([
    mlContext.createTensor({
      dataType: 'float32',
      dimensions: [4],
      usage: MLTensorUsage.WRITE | MLTensorUsage.READ
    }),
    mlContext.createTensor(
        {dataType: 'float32', dimensions: [4], usage: MLTensorUsage.READ}),
    builder.build({'output': outputOperand})
  ]);

  const inputData = Float32Array.from([-4, -1, 1, 4]);
  mlContext.writeTensor(inputTensor, inputData);

  mlContext.dispatch(mlGraph, {'input': inputTensor}, {'output': outputTensor});

  // Wait for graph execution to complete.
  await mlContext.readTensor(outputTensor);

  // The input tensor should not be modified.
  assert_array_equals(
      new Float32Array(await mlContext.readTensor(inputTensor)), inputData);
}, 'input tensor is not modified: hardSwish');

promise_test(async () => {
  const builder = new MLGraphBuilder(mlContext);
  const inputOperand =
      builder.input('input', {dataType: 'float32', dimensions: [4]});
  const constantOperand = builder.constant(
      {dataType: 'float32', dimensions: [4]}, Float32Array.from([-2, 0, 3, 4]));
  const mulOperand = builder.mul(inputOperand, constantOperand);
  // Add some other operator for the output tensor to bind to; otherwise there
  // is no reason to implement mul "in-place".
  const outputOperand = builder.add(mulOperand, constantOperand);

  const [inputTensor, outputTensor, mlGraph] = await Promise.all([
    mlContext.createTensor({
      dataType: 'float32',
      dimensions: [4],
      usage: MLTensorUsage.WRITE | MLTensorUsage.READ
    }),
    mlContext.createTensor(
        {dataType: 'float32', dimensions: [4], usage: MLTensorUsage.READ}),
    builder.build({'output': outputOperand})
  ]);

  const inputData = Float32Array.from([1, 2, 3, 4]);
  mlContext.writeTensor(inputTensor, inputData);
  mlContext.dispatch(mlGraph, {'input': inputTensor}, {'output': outputTensor});

  // Wait for graph execution to complete.
  await mlContext.readTensor(outputTensor);

  // The input tensor should not be modified.
  assert_array_equals(
      new Float32Array(await mlContext.readTensor(inputTensor)), inputData);
}, 'input tensor is not modified: mul');
