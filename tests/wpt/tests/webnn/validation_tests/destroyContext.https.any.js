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
    const operandType = {dataType: 'float32', shape: [1]};
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
  const operandType = {dataType: 'float32', shape: [1]};
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
  const operandType = {dataType: 'float32', shape: [1]};
  const lhsOperand = builder.input('lhs', operandType);
  const rhsOperand = builder.input('rhs', operandType);
  const graph =
      await builder.build({'output': builder.sub(lhsOperand, rhsOperand)});

  const lhsTensor = await context.createTensor(operandType);
  const rhsTensor = await context.createTensor(operandType);

  const dispatchOutputs = {'output': await context.createTensor(operandType)};
  context.destroy();
  assert_throws_dom('InvalidStateError', () => {
    context.dispatch(
        graph, {
          'lhs': lhsTensor,
          'rhs': rhsTensor,
        },
        dispatchOutputs);
  });
}, 'Destroyed context can not dispatch.');

promise_test(async t => {
  const context = await navigator.ml.createContext(contextOptions);
  const builder = new MLGraphBuilder(context);
  const operandType = {dataType: 'float32', shape: [1]};
  const lhsOperand = builder.input('lhs', operandType);
  const rhsOperand = builder.input('rhs', operandType);
  const graph =
      await builder.build({'output': builder.sub(lhsOperand, rhsOperand)});

  const lhsTensor = await context.createTensor(operandType);
  const rhsTensor = await context.createTensor(operandType);

  const dispatchOutputs = {'output': await context.createTensor(operandType)};
  context.dispatch(
      graph, {
        'lhs': lhsTensor,
        'rhs': rhsTensor,
      },
      dispatchOutputs);
  context.destroy();
}, 'Executing dispatch() before context destroyed is OK.');

promise_test(async t => {
  const context = await navigator.ml.createContext(contextOptions);
  context.destroy();
  promise_rejects_dom(
      t, 'InvalidStateError',
      context.createTensor({dataType: 'float32', shape: [1]}));
}, 'Destroyed context can not create tensor.');

promise_test(async t => {
  const context = await navigator.ml.createContext(contextOptions);
  const tensor = await context.createTensor({
    dataType: 'float32',
    shape: [1],
    readable: true,
  });
  context.destroy();
  promise_rejects_dom(t, 'InvalidStateError', context.readTensor(tensor));
}, 'Destroyed context can not read tensor.');

promise_test(async t => {
  const context = await navigator.ml.createContext(contextOptions);
  const tensor = await context.createTensor({
    dataType: 'float32',
    shape: [1],
    readable: true,
  });
  let promise = context.readTensor(tensor);
  context.destroy();
  promise_rejects_dom(t, 'InvalidStateError', promise);
}, 'Pending promise of readtensor() will be rejected immediately when context is destroyed.');

promise_test(async t => {
  const context = await navigator.ml.createContext(contextOptions);
  // Destroying another context doesn't impact the first context.
  const another_context = await navigator.ml.createContext(contextOptions);
  another_context.destroy();
  const tensor = await context.createTensor({
    dataType: 'float32',
    shape: [1],
    writable: true,
  });
  let arrayBuffer = new ArrayBuffer(4);
  context.destroy();
  assert_throws_dom('InvalidStateError', () => {
    context.writeTensor(tensor, new Uint8Array(arrayBuffer));
  });
}, 'Destroyed context can not write tensor.');
