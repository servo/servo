// META: title=ensure MLMLGraphBuilder may build at most one MLGraph
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

const kExampleInputDescriptor = {
  dataType: 'float32',
  shape: [2]
};

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const a = builder.input('a', kExampleInputDescriptor);
  const b = builder.input('b', kExampleInputDescriptor);
  const c = builder.add(a, b);
  const graph = await builder.build({c});

  await promise_rejects_dom(t, 'InvalidStateError', builder.build({c}));
}, 'Throw if attempting to build a second graph with an MLGraphBuilder');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const a = builder.input('a', kExampleInputDescriptor);
  const b = builder.input('b', kExampleInputDescriptor);
  const c = builder.add(a, b);
  const graph_promise_not_awaited = builder.build({c});

  await promise_rejects_dom(t, 'InvalidStateError', builder.build({c}));
}, 'Throw if attempting to build a second graph without awaiting the first');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const a = builder.input('a', kExampleInputDescriptor);
  const b = builder.input('b', kExampleInputDescriptor);
  const c = builder.add(a, b);
  const graph = await builder.build({c});

  assert_throws_dom('InvalidStateError', () => builder.sub(a, b));
}, 'Throw if an operand-yielding method is called on a built MLGraphBuilder');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const a = builder.input('a', kExampleInputDescriptor);
  const b = builder.input('b', kExampleInputDescriptor);
  const c = builder.add(a, b);
  const graph = await builder.build({c});

  assert_throws_dom(
      'InvalidStateError', () => builder.input('d', kExampleInputDescriptor));
}, 'Throw if adding an input operand to a built MLGraphBuilder');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const a = builder.input('a', kExampleInputDescriptor);
  const b = builder.input('b', kExampleInputDescriptor);
  const c = builder.add(a, b);
  const graph = await builder.build({c});

  const buffer = new ArrayBuffer(8);
  const bufferView = new Float32Array(buffer);

  assert_throws_dom(
      'InvalidStateError',
      () => builder.constant(kExampleInputDescriptor, bufferView));
}, 'Throw if adding a constant operand to a built MLGraphBuilder');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const a = builder.input('a', kExampleInputDescriptor);
  const b = builder.input('b', kExampleInputDescriptor);
  const c = builder.add(a, b);

  // Call build() with invalid parameters.
  await promise_rejects_js(t, TypeError, builder.build({a}));

  // Passing valid parameters successfully creates the graph...
  const graph = await builder.build({c});

  // ...exactly once!
  await promise_rejects_dom(t, 'InvalidStateError', builder.build({c}));
}, 'An MLGraphBuilder remains unbuilt if build() is called with invalid paramaters');

promise_test(async t => {
  const builder1 = new MLGraphBuilder(context);
  const builder2 = new MLGraphBuilder(context);

  const a1 = builder1.input('a', kExampleInputDescriptor);
  const b1 = builder1.input('b', kExampleInputDescriptor);
  const c1 = builder1.add(a1, b1);
  const graph1 = await builder1.build({c1});

  const a2 = builder2.input('a', kExampleInputDescriptor);
  const b2 = builder2.input('b', kExampleInputDescriptor);
  const c2 = builder2.add(a2, b2);
  const graph2 = await builder2.build({c2});
}, 'Build two graphs with separate MLGraphBuilders');
