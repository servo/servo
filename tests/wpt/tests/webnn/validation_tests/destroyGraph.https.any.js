// META: timeout=long
// META: title=validation tests for WebNN API MLContext::destroy()
// META: global=window
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
    writable: true,
  });
  const rhsTensor = await context.createTensor({
    dataType: 'float32',
    shape: [1],
    writable: true,
  });
  const outputTensor = await context.createTensor({
    dataType: 'float32',
    shape: [1],
    readable: true,
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

promise_test(async t => {
  const operandType = {dataType: 'float32', shape: [1]};

  const builder1 = new MLGraphBuilder(context);
  const lhsOperand1 = builder1.input('lhs', operandType);
  const rhsOperand1 = builder1.input('rhs', operandType);
  const graph1 =
      await builder1.build({'output': builder1.mul(lhsOperand1, rhsOperand1)});
  t.add_cleanup(() => graph1.destroy());

  const builder2 = new MLGraphBuilder(context);
  const lhsOperand2 = builder2.input('lhs', operandType);
  const rhsOperand2 = builder2.input('rhs', operandType);
  const graph2 =
      await builder2.build({'output': builder2.add(lhsOperand2, rhsOperand2)});

  graph2.destroy();

  const lhsTensor = await context.createTensor({
    dataType: 'float32',
    shape: [1],
    writable: true,
  });
  const rhsTensor = await context.createTensor({
    dataType: 'float32',
    shape: [1],
    writable: true,
  });
  const outputTensor = await context.createTensor({
    dataType: 'float32',
    shape: [1],
    readable: true,
  });
  t.add_cleanup(() => {
    lhsTensor.destroy();
    rhsTensor.destroy();
    outputTensor.destroy();
  });

  const inputData = new Float32Array(1).fill(3.0);
  context.writeTensor(lhsTensor, inputData);
  context.writeTensor(rhsTensor, inputData);
  const dispatchInputs = {'lhs': lhsTensor, 'rhs': rhsTensor};
  const dispatchOutputs = {'output': outputTensor};
  context.dispatch(graph1, dispatchInputs, dispatchOutputs);

  const outputData = await context.readTensor(outputTensor);
  assert_array_equals(
      new Float32Array(outputData), [9],
      'Read tensor data equals expected data.');
}, 'Destroying another graph on the same context before dispatch() is OK.');
