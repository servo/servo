// META: title=test that scalar values work as expected
// META: global=window,worker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

let mlContext;

// Skip tests if WebNN is unimplemented.
promise_setup(async () => {
  assert_implements(navigator.ml, 'missing navigator.ml');
  mlContext = await navigator.ml.createContext(contextOptions);
});

promise_test(async () => {
  const builder = new MLGraphBuilder(mlContext);
  const inputOperand = builder.input('input', {dataType: 'int32', shape: []});
  const constantOperand = builder.constant(
      {dataType: 'int32', shape: [4]}, Int32Array.from([3, 2, 1, 7]));
  const addOperand = builder.add(inputOperand, constantOperand);

  const [inputTensor, outputTensor, mlGraph] = await Promise.all([
    mlContext.createTensor({dataType: 'int32', shape: [], writable: true}),
    mlContext.createTensor({dataType: 'int32', shape: [4], readable: true}),
    builder.build({'output': addOperand})
  ]);

  mlContext.writeTensor(inputTensor, Int32Array.from([4]));
  mlContext.dispatch(mlGraph, {'input': inputTensor}, {'output': outputTensor});
  assert_array_equals(
      new Int32Array(await mlContext.readTensor(outputTensor)),
      Int32Array.from([7, 6, 5, 11]));
}, 'scalar input');

promise_test(async () => {
  const builder = new MLGraphBuilder(mlContext);
  const inputOperand = builder.input('input', {dataType: 'float32', shape: []});
  const constantOperand = builder.constant(
      {dataType: 'float32', shape: []}, Float32Array.from([3]));
  const addOperand = builder.add(inputOperand, constantOperand);

  const [inputTensor, outputTensor, mlGraph] = await Promise.all([
    mlContext.createTensor({dataType: 'float32', shape: [], writable: true}),
    mlContext.createTensor({dataType: 'float32', shape: [], readable: true}),
    builder.build({'output': addOperand})
  ]);

  mlContext.writeTensor(inputTensor, Float32Array.from([4]));

  mlContext.dispatch(mlGraph, {'input': inputTensor}, {'output': outputTensor});

  assert_array_equals(
      new Float32Array(await mlContext.readTensor(outputTensor)),
      Float32Array.from([7]));
}, 'float32 scalar input, constant, and output');

promise_test(async () => {
  const builder = new MLGraphBuilder(mlContext);
  const inputOperand = builder.input('input', {dataType: 'int32', shape: []});
  const constantOperand =
      builder.constant({dataType: 'int32', shape: []}, Int32Array.from([3]));
  const addOperand = builder.add(inputOperand, constantOperand);

  const [inputTensor, outputTensor, mlGraph] = await Promise.all([
    mlContext.createTensor({dataType: 'int32', shape: [], writable: true}),
    mlContext.createTensor({dataType: 'int32', shape: [], readable: true}),
    builder.build({'output': addOperand})
  ]);

  mlContext.writeTensor(inputTensor, Int32Array.from([4]));
  mlContext.dispatch(mlGraph, {'input': inputTensor}, {'output': outputTensor});
  assert_array_equals(
      new Int32Array(await mlContext.readTensor(outputTensor)),
      Int32Array.from([7]));
}, 'int32 scalar input, constant, and output');
