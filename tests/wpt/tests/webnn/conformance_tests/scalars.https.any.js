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

// Tests for constant(type, value)
promise_test(async () => {
  const builder = new MLGraphBuilder(mlContext);
  const inputOperand = builder.input('input', {dataType: 'float32', shape: []});
  const constantOperand = builder.constant('float32', 3.0);
  const addOperand = builder.add(inputOperand, constantOperand);

  const [inputTensor, outputTensor, mlGraph] = await Promise.all([
    mlContext.createTensor({dataType: 'float32', shape: [], writable: true}),
    mlContext.createTensor({dataType: 'float32', shape: [], readable: true}),
    builder.build({'output': addOperand})
  ]);

  mlContext.writeTensor(inputTensor, Float32Array.from([2.0]));
  mlContext.dispatch(mlGraph, {'input': inputTensor}, {'output': outputTensor});

  const result = new Float32Array(await mlContext.readTensor(outputTensor));
  assert_array_equals(result, Float32Array.from([5.0]));
}, 'scalar constant created with constant(type, value) - float32');

promise_test(async () => {
  const builder = new MLGraphBuilder(mlContext);
  const inputOperand = builder.input('input', {dataType: 'int32', shape: []});
  const constantOperand = builder.constant('int32', 42);
  const mulOperand = builder.mul(inputOperand, constantOperand);

  const [inputTensor, outputTensor, mlGraph] = await Promise.all([
    mlContext.createTensor({dataType: 'int32', shape: [], writable: true}),
    mlContext.createTensor({dataType: 'int32', shape: [], readable: true}),
    builder.build({'output': mulOperand})
  ]);

  mlContext.writeTensor(inputTensor, Int32Array.from([3]));
  mlContext.dispatch(mlGraph, {'input': inputTensor}, {'output': outputTensor});
  assert_array_equals(
      new Int32Array(await mlContext.readTensor(outputTensor)),
      Int32Array.from([126]));
}, 'scalar constant created with constant(type, value) - int32');

promise_test(async () => {
  const builder = new MLGraphBuilder(mlContext);
  const inputOperand = builder.input('input', {dataType: 'float16', shape: [3]});
  const zeroConstant = builder.constant('float16', 2.0);
  const negativeConstant = builder.constant('float16', -1.0);

  // Test complex expression: input * 2 + (-1.0)
  const mulResult = builder.mul(inputOperand, zeroConstant);
  const addResult = builder.add(mulResult, negativeConstant);

  const [inputTensor, outputTensor, mlGraph] = await Promise.all([
    mlContext.createTensor({dataType: 'float16', shape: [3], writable: true}),
    mlContext.createTensor({dataType: 'float16', shape: [3], readable: true}),
    builder.build({'output': addResult})
  ]);

  mlContext.writeTensor(inputTensor, Float16Array.from([1.0, 2.0, 3.0]));
  mlContext.dispatch(mlGraph, {'input': inputTensor}, {'output': outputTensor});

  const result = new Float16Array(await mlContext.readTensor(outputTensor));
  assert_array_equals(result, Float16Array.from([1.0, 3.0, 5.0]));
}, 'multiple scalar constants in expression - float16');
