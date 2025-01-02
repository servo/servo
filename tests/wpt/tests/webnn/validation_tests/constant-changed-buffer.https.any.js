// META: title=ensure MLGraphBuilder.constant() handles buffers which change
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  let backingBuffer = new ArrayBuffer(8);
  let aBuffer = new Float32Array(backingBuffer, 0, 2);
  aBuffer[0] = 2;
  aBuffer[1] = 3;
  const a = builder.constant({dataType: 'float32', shape: [2]}, aBuffer);

  // Detach `aBuffer`. Constant data should already be copied, so changes to
  // the buffer afterwards should not be reflected in the graph.
  const unusedBuffer = backingBuffer.transfer();

  const b = builder.input('b', {dataType: 'float32', shape: [2]});
  const c = builder.add(a, b);

  const [graph, bTensor, cTensor] = await Promise.all([
    builder.build({c}),
    context.createTensor({dataType: 'float32', shape: [2], writable: true}),
    context.createTensor({dataType: 'float32', shape: [2], readable: true})
  ]);

  context.writeTensor(bTensor, new Float32Array([5, 7]));

  context.dispatch(graph, {'b': bTensor}, {'c': cTensor});
  const result = new Float32Array(await context.readTensor(cTensor));
  assert_array_equals(result, new Float32Array([7, 10]));
}, 'Constant data is unaffected by detaching the buffer');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  let aBuffer = new Float32Array([2, 3]);
  const a = builder.constant({dataType: 'float32', shape: [2]}, aBuffer);

  // Rewrite `aBuffer` contents. Constant data should already be copied, so
  // changes to the buffer afterwards should not be reflected in the graph.
  aBuffer[0] = 10;
  aBuffer[1] = 20;

  const b = builder.input('b', {dataType: 'float32', shape: [2]});
  const c = builder.add(a, b);

  const [graph, bTensor, cTensor] = await Promise.all([
    builder.build({c}),
    context.createTensor({dataType: 'float32', shape: [2], writable: true}),
    context.createTensor({dataType: 'float32', shape: [2], readable: true})
  ]);

  context.writeTensor(bTensor, new Float32Array([5, 7]));

  context.dispatch(graph, {'b': bTensor}, {'c': cTensor});
  const result = new Float32Array(await context.readTensor(cTensor));
  assert_array_equals(result, new Float32Array([7, 10]));
}, 'Constant data is unaffected by changes to the buffer contents');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  let backingBuffer = new ArrayBuffer(8);
  const aBuffer = new Float32Array(backingBuffer, 0, 2);
  // Detach `aBuffer` _before_ calling `constant()`. This should throw, since
  // detached buffers have a length of zero, which does not match the length of
  // the descriptor. See
  // https://webidl.spec.whatwg.org/#dfn-get-buffer-source-copy
  const unusedBuffer = backingBuffer.transfer();

  assert_throws_js(
      TypeError,
      () => builder.constant({dataType: 'float32', shape: [2]}, aBuffer));
}, 'Constant data cannot use a detached buffer, which is empty');
