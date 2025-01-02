// META: title=test graph inputs/outputs with unprintable names
// META: global=window,dedicatedworker
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
  const operandDescriptor = {
    dataType: 'float32',
    shape: [1],
  };

  // Construct a simple graph: A = B * 2.
  const builder = new MLGraphBuilder(mlContext);
  const inputOperand = builder.input('input\x00tensor', operandDescriptor);
  const constantOperand =
      builder.constant(operandDescriptor, Float32Array.from([2]));
  const outputOperand = builder.mul(inputOperand, constantOperand);
  const mlGraph = await builder.build({'output\x00tensor': outputOperand});

  const [inputTensor, outputTensor] = await Promise.all([
    mlContext.createTensor({dataType: 'float32', shape: [1], writable: true}),
    mlContext.createTensor({dataType: 'float32', shape: [1], readable: true})
  ]);

  mlContext.writeTensor(inputTensor, Float32Array.from([1]));

  mlContext.dispatch(
      mlGraph, {'input\x00tensor': inputTensor},
      {'output\x00tensor': outputTensor});

  const output = await mlContext.readTensor(outputTensor);
  assert_equals(new Float32Array(output)[0], 2);
}, 'tensor names can include null bytes');
