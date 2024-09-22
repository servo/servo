// META: title=test WebNN MLContext.compute() for ArrayBufferView created from bigger ArrayBuffer
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js

'use strict';

// These tests are used to reproduce the Chromium issue:
// https://issues.chromium.org/issues/332151809

if (navigator.ml) {
  const variant = location.search.substring(1);
  const contextOptions = kContextOptionsForVariant[variant];

  let context;

  promise_setup(async () => {
    try {
      context = await navigator.ml.createContext(contextOptions);
    } catch (e) {
      throw new AssertionError(
          `Unable to create context for ${variant} variant. ${e}`);
    }
  });

  promise_test(async t => {
    const builder = new MLGraphBuilder(context);
    const a = builder.input('a', {dataType: 'float32', shape: [2]});
    const b = builder.relu(a);
    const graph = await builder.build({b});
    const arraybuffer = new ArrayBuffer(100);
    const aBuffer =
        new Float32Array(arraybuffer, /*byteOffset*/ 4, /*length*/ 2)
    aBuffer.set([1, -1]);
    const bBuffer = new Float32Array(2);
    const results =
        await context.compute(graph, {'a': aBuffer}, {'b': bBuffer});
    assert_array_approx_equals_ulp(
        results.outputs.b, [1, 0], /*nulp*/ 0, 'float32');
  }, 'Test compute() working for input ArrayBufferView created from bigger ArrayBuffer');

  promise_test(async t => {
    const builder = new MLGraphBuilder(context);
    const a = builder.input('a', {dataType: 'float32', shape: [2]});
    const b = builder.relu(a);
    const graph = await builder.build({b});
    const aBuffer = new Float32Array(2);
    aBuffer.set([1, -1]);
    const arraybuffer = new ArrayBuffer(100);
    const bBuffer =
        new Float32Array(arraybuffer, /*byteOffset*/ 8, /*length*/ 2);
    const results =
        await context.compute(graph, {'a': aBuffer}, {'b': bBuffer});
    assert_array_approx_equals_ulp(
        results.outputs.b, [1, 0], /*nulp*/ 0, 'float32');
  }, 'Test compute() working for output ArrayBufferView created from bigger ArrayBuffer');
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
