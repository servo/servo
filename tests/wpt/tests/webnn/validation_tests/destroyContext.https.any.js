// META: timeout=long
// META: title=validation tests for WebNN API MLContext::destroy()
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu

'use strict';

let contextOptions;

promise_setup(async () => {
  if (navigator.ml === undefined) {
    return;
  }
  contextOptions = {deviceType: location.search.substring(1)};
}, {explicit_timeout: true});

promise_test(async t => {
  const context = await navigator.ml.createContext(contextOptions);
  context.destroy();
  await context.lost;
}, 'Context will be lost by destroyed.');

promise_test(async t => {
  const context = await navigator.ml.createContext(contextOptions);
  context.destroy();
  context.destroy();
  await context.lost;
}, 'Context can be destroyed twice.');

promise_test(async t => {
  const context = await navigator.ml.createContext(contextOptions);
  const builder = new MLGraphBuilder(context);
  context.destroy();
  assert_throws_dom('InvalidStateError', () => {
    const operandType = {dataType: 'float32', dimensions: [1]};
    builder.input('input', operandType);
  });
}, 'Destroyed context can not build operator.');

promise_test(async t => {
  const context = await navigator.ml.createContext(contextOptions);
  context.destroy();
  assert_throws_dom('InvalidStateError', () => {
    new MLGraphBuilder(context);
  });
}, 'Destroyed context can not create graph builder.');

promise_test(async t => {
  const context = await navigator.ml.createContext(contextOptions);
  const builder = new MLGraphBuilder(context);
  const operandType = {dataType: 'float32', dimensions: [1]};
  const input_operand = builder.input('input', operandType);
  const const_operand = builder.constant(operandType, Float32Array.from([2]));
  const output_operand = builder.mul(input_operand, const_operand);

  context.destroy();
  promise_rejects_dom(
      t, 'InvalidStateError', builder.build({'output': output_operand}));
}, 'Destroyed context can not build graph.');

promise_test(async t => {
  const context = await navigator.ml.createContext(contextOptions);
  const builder = new MLGraphBuilder(context);
  const operandType = {dataType: 'float32', dimensions: [1]};
  const input_operand = builder.input('input', operandType);
  const const_operand = builder.constant(operandType, Float32Array.from([2]));
  const output_operand = builder.mul(input_operand, const_operand);
  const graph = await builder.build({'output': output_operand});

  context.destroy();
  let inputs = {'input': Float32Array.from([1])};
  let outputs = {'output': new Float32Array(1)};
  promise_rejects_dom(
      t, 'InvalidStateError', context.compute(graph, inputs, outputs));
}, 'Destroyed context can not compute.');

promise_test(async t => {
  const context = await navigator.ml.createContext(contextOptions);
  const builder = new MLGraphBuilder(context);
  const operandType = {dataType: 'float32', dimensions: [1]};
  const lhsOperand = builder.input('lhs', operandType);
  const rhsOperand = builder.input('rhs', operandType);
  const graph =
      await builder.build({'output': builder.sub(lhsOperand, rhsOperand)});

  const lhsBuffer = await context.createBuffer(operandType);
  const rhsBuffer = await context.createBuffer(operandType);

  const dispatchOutputs = {'output': await context.createBuffer(operandType)};
  context.destroy();
  assert_throws_dom('InvalidStateError', () => {
    context.dispatch(
        graph, {
          'lhs': lhsBuffer,
          'rhs': rhsBuffer,
        },
        dispatchOutputs);
  });
}, 'Destroyed context can not dispatch.');

promise_test(async t => {
  const context = await navigator.ml.createContext(contextOptions);
  const builder = new MLGraphBuilder(context);
  const operandType = {dataType: 'float32', dimensions: [1]};
  const lhsOperand = builder.input('lhs', operandType);
  const rhsOperand = builder.input('rhs', operandType);
  const graph =
      await builder.build({'output': builder.sub(lhsOperand, rhsOperand)});

  const lhsBuffer = await context.createBuffer(operandType);
  const rhsBuffer = await context.createBuffer(operandType);

  const dispatchOutputs = {'output': await context.createBuffer(operandType)};
  context.dispatch(
      graph, {
        'lhs': lhsBuffer,
        'rhs': rhsBuffer,
      },
      dispatchOutputs);
  context.destroy();
}, 'Executing dispatch() before context destroyed is OK.');

promise_test(async t => {
  const context = await navigator.ml.createContext(contextOptions);
  context.destroy();
  promise_rejects_dom(
      t, 'InvalidStateError',
      context.createBuffer({dataType: 'float32', dimensions: [1]}));
}, 'Destroyed context can not create buffer.');

promise_test(async t => {
  const context = await navigator.ml.createContext(contextOptions);
  const buffer = await context.createBuffer({
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.READ_FROM,
  });
  context.destroy();
  promise_rejects_dom(t, 'InvalidStateError', context.readBuffer(buffer));
}, 'Destroyed context can not read buffer.');

promise_test(async t => {
  const context = await navigator.ml.createContext(contextOptions);
  const buffer = await context.createBuffer({
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.READ_FROM,
  });
  let promise = context.readBuffer(buffer);
  context.destroy();
  promise_rejects_dom(t, 'InvalidStateError', promise);
}, 'Pending promise of readbuffer() will be rejected immediately when context is destroyed.');

promise_test(async t => {
  const context = await navigator.ml.createContext(contextOptions);
  // Destroying another context doesn't impact the first context.
  const another_context = await navigator.ml.createContext(contextOptions);
  another_context.destroy();
  const buffer = await context.createBuffer({
    dataType: 'float32',
    dimensions: [1],
    usage: MLTensorUsage.WRITE_TO,
  });
  let arrayBuffer = new ArrayBuffer(4);
  context.destroy();
  assert_throws_dom('InvalidStateError', () => {
    context.writeBuffer(buffer, new Uint8Array(arrayBuffer));
  });
}, 'Destroyed context can not write buffer.');
