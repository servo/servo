// META: timeout=long
// META: title=validation tests for WebNN API MLContext::destroy()
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu

'use strict';

let context;

promise_setup(async () => {
  assert_implements(navigator.ml, 'WebNN is not supported');
  const contextOptions = {deviceType: location.search.substring(1)};
  try {
    context = await navigator.ml.createContext(contextOptions);
  } catch (e) {
    throw new AssertionError(
        `Unable to create context for ${variant} variant. ${e}`);
  }
}, {explicit_timeout: true});

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const operandType = {dataType: 'float32', shape: [1]};
  const input_operand = builder.input('input', operandType);
  const const_operand = builder.constant(operandType, Float32Array.from([2]));
  const output_operand = builder.mul(input_operand, const_operand);
  const graph = await builder.build({'output': output_operand});

  graph.destroy();
  graph.destroy();
}, 'Graph can be destroyed twice.');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const operandType = {dataType: 'float32', shape: [1]};
  const input_operand = builder.input('input', operandType);
  const const_operand = builder.constant(operandType, Float32Array.from([2]));
  const output_operand = builder.mul(input_operand, const_operand);
  const graph = await builder.build({'output': output_operand});

  graph.destroy();
  let inputs = {'input': Float32Array.from([1])};
  let outputs = {'output': new Float32Array(1)};
  promise_rejects_dom(
      t, 'InvalidStateError', context.compute(graph, inputs, outputs));
}, 'Destroyed graph can not compute.');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const operandType = {dataType: 'float32', shape: [1]};
  const input_operand = builder.input('input', operandType);
  const const_operand = builder.constant(operandType, Float32Array.from([2]));
  const output_operand = builder.mul(input_operand, const_operand);
  const graph = await builder.build({'output': output_operand});

  let inputs = {'input': Float32Array.from([1])};
  let outputs = {'output': new Float32Array(1)};
  await context.compute(graph, inputs, outputs);
  graph.destroy();
}, 'Destroying graph after compute() with await is OK.');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const operandType = {dataType: 'float32', shape: [1]};
  const input_operand = builder.input('input', operandType);
  const const_operand = builder.constant(operandType, Float32Array.from([2]));
  const output_operand = builder.mul(input_operand, const_operand);
  const graph = await builder.build({'output': output_operand});

  let inputs = {'input': Float32Array.from([1])};
  let outputs = {'output': new Float32Array(1)};
  const promise = context.compute(graph, inputs, outputs);
  graph.destroy();
  promise_rejects_dom(t, 'InvalidStateError', promise);
}, 'compute() rejects when graph is destroyed.');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const operandType = {dataType: 'float32', shape: [1]};
  const lhsOperand = builder.input('lhs', operandType);
  const rhsOperand = builder.input('rhs', operandType);
  const graph =
      await builder.build({'output': builder.mul(lhsOperand, rhsOperand)});

  const lhsTensor = await context.createTensor(operandType);
  const rhsTensor = await context.createTensor(operandType);
  const dispatchOutputs = {'output': await context.createTensor(operandType)};

  graph.destroy();
  assert_throws_dom('InvalidStateError', () => {
    context.dispatch(
        graph, {
          'lhs': lhsTensor,
          'rhs': rhsTensor,
        },
        dispatchOutputs);
  });
}, 'Destroyed graph can not dispatch.');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const operandType = {dataType: 'float32', shape: [1]};
  const lhsOperand = builder.input('lhs', operandType);
  const rhsOperand = builder.input('rhs', operandType);
  const graph =
      await builder.build({'output': builder.mul(lhsOperand, rhsOperand)});

  const lhsTensor = await context.createTensor({
    dataType: 'float32',
    shape: [1],
    usage: MLTensorUsage.WRITE,
  });
  const rhsTensor = await context.createTensor({
    dataType: 'float32',
    shape: [1],
    usage: MLTensorUsage.WRITE,
  });
  const outputTensor = await context.createTensor({
    dataType: 'float32',
    shape: [1],
    usage: MLTensorUsage.READ,
  });
  // Initialize inputs
  const inputData = new Float32Array(1).fill(2.0);
  context.writeTensor(lhsTensor, inputData);
  context.writeTensor(rhsTensor, inputData);
  context.dispatch(
      graph, {
        'lhs': lhsTensor,
        'rhs': rhsTensor,
      },
      {'output': outputTensor});

  graph.destroy();
  const outputData = await context.readTensor(outputTensor);
  assert_array_equals(
      new Float32Array(outputData), [4],
      'Read tensor data equals expected data.');
}, 'Destroying graph after dispatch() and before readTensor() is OK.');
