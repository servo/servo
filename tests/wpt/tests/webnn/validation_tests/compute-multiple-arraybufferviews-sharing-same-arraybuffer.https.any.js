// META: title=ensure WebNN MLContext.compute() rejecting detached buffers
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

// These tests are used to reproduce the Chromium issue:
// https://issues.chromium.org/issues/332002364
promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const a = builder.input('a', {dataType: 'float32', shape: [2]});
  const b = builder.input('b', {dataType: 'float32', shape: [2]});
  const c = builder.add(a, b);
  const graph = await builder.build({c});
  const arraybuffer = new ArrayBuffer(100);
  const aBuffer = new Float32Array(arraybuffer, 0, 2);
  const bBuffer = new Float32Array(arraybuffer, 8, 2);
  const cBuffer = new Float32Array(2);
  const promise =
      context.compute(graph, {'a': aBuffer, 'b': bBuffer}, {'c': cBuffer});
  promise_rejects_js(t, TypeError, promise);
}, 'Throw if two input ArrayBufferViews sharing the same ArrayBuffer');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const a = builder.input('a', {dataType: 'float32', shape: [2]});
  const [b, c] = builder.split(a, 2);
  const graph = await builder.build({b, c});
  const aBuffer = new Float32Array(2);
  const arraybuffer = new ArrayBuffer(100);
  const bBuffer = new Float32Array(arraybuffer, 0, 1);
  const cBuffer = new Float32Array(arraybuffer, 4, 1);
  const promise =
      context.compute(graph, {'a': aBuffer}, {'b': bBuffer, 'c': cBuffer});
  promise_rejects_js(t, TypeError, promise);
}, 'Throw if two output ArrayBufferViews sharing the same ArrayBuffer');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const a = builder.input('a', {dataType: 'float32', shape: [2]});
  const b = builder.relu(a);
  const graph = await builder.build({b});
  const arraybuffer = new ArrayBuffer(100);
  const aBuffer = new Float32Array(arraybuffer, 0, 2);
  const bBuffer = new Float32Array(arraybuffer, 8, 2);
  const promise = context.compute(graph, {'a': aBuffer}, {'b': bBuffer});
  promise_rejects_js(t, TypeError, promise);
}, 'Throw if input and output ArrayBufferViews sharing the same ArrayBuffer');

promise_test(async t => {
  const builder = new MLGraphBuilder(context);
  const a = builder.input('a', {dataType: 'float32', shape: [2]});
  const b = builder.relu(a);
  const graph = await builder.build({b});
  const buffer = new Float32Array(2);
  const promise = context.compute(graph, {'a': buffer}, {'b': buffer});
  promise_rejects_js(t, TypeError, promise);
}, 'Throw if input and output are the same ArrayBufferView');
