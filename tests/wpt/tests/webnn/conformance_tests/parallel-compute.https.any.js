// META: title=test parallel WebNN API compute operations
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlcontext-compute

/**
 * WebNN parallel compute operation test.
 */
const testParallelCompute = () => {
  let mlContext;
  let mlGraph;

  promise_setup(async () => {
    try {
      mlContext = await navigator.ml.createContext(contextOptions);
    } catch (e) {
      throw new AssertionError(
          `Unable to create context for ${variant} variant. ${e}`);
    }
    // Construct a simple graph: A = B * 2.
    const builder = new MLGraphBuilder(mlContext);
    const operandType = {dataType: 'float32', shape: [1]};
    const inputOperand = builder.input('input', operandType);
    const constOperand = builder.constant(operandType, Float32Array.from([2]));
    const outputOperand = builder.mul(inputOperand, constOperand);
    mlGraph = await builder.build({'output': outputOperand});
  });

  promise_test(async () => {
    const testInputs = [1, 2, 3, 4];

    const actualOutputs = await Promise.all(testInputs.map(async (input) => {
      let inputs = {'input': Float32Array.from([input])};
      let outputs = {'output': new Float32Array(1)};
      ({inputs, outputs} = await mlContext.compute(mlGraph, inputs, outputs));
      return outputs.output[0];
    }));

    const expectedOutputs = [2, 4, 6, 8];
    assert_array_equals(actualOutputs, expectedOutputs);
  });
};

if (navigator.ml) {
  testParallelCompute();
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
