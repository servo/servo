'use strict';

if (self.importScripts) {
  self.importScripts('/resources/testharness.js');
}

const error1 = new Error('error1');
error1.name = 'error1';

promise_test(() => {
  let controller;
  const ws = new WritableStream({
    start(c) {
      controller = c;
    }
  });

  // Now error the stream after its construction.
  controller.error(error1);

  const writer = ws.getWriter();

  assert_equals(writer.desiredSize, null, 'desiredSize should be null');
  return writer.closed.catch(r => {
    assert_equals(r, error1, 'ws should be errored by the passed error');
  });
}, 'controller argument should be passed to start method');

promise_test(t => {
  const ws = new WritableStream({
    write(chunk, controller) {
      controller.error(error1);
    }
  });

  const writer = ws.getWriter();

  return Promise.all([
    writer.write('a'),
    promise_rejects(t, error1, writer.closed, 'controller.error() in write() should errored the stream')
  ]);
}, 'controller argument should be passed to write method');

promise_test(t => {
  const ws = new WritableStream({
    close(controller) {
      controller.error(error1);
    }
  });

  const writer = ws.getWriter();

  return Promise.all([
    writer.close(),
    promise_rejects(t, error1, writer.closed, 'controller.error() in close() should error the stream')
  ]);
}, 'controller argument should be passed to close method');

promise_test(() => {
  const ws = new WritableStream({}, {
    highWaterMark: 1000,
    size() { return 1; }
  });

  const writer = ws.getWriter();

  assert_equals(writer.desiredSize, 1000, 'desiredSize should be 1000');
  return writer.ready.then(v => {
    assert_equals(v, undefined, 'ready promise should fulfill with undefined');
  });
}, 'highWaterMark should be reflected to desiredSize');

promise_test(() => {
  const ws = new WritableStream({}, {
    highWaterMark: Infinity,
    size() { return 0; }
  });

  const writer = ws.getWriter();

  assert_equals(writer.desiredSize, Infinity, 'desiredSize should be Infinity');

  return writer.ready;
}, 'WritableStream should be writable and ready should fulfill immediately if the strategy does not apply ' +
    'backpressure');

test(() => {
  new WritableStream();
}, 'WritableStream should be constructible with no arguments');

test(() => {
  const ws = new WritableStream({});

  const writer = ws.getWriter();

  assert_equals(typeof writer.write, 'function', 'writer should have a write method');
  assert_equals(typeof writer.abort, 'function', 'writer should have an abort method');
  assert_equals(typeof writer.close, 'function', 'writer should have a close method');

  assert_equals(writer.desiredSize, 1, 'desiredSize should start at 1');

  assert_not_equals(typeof writer.ready, 'undefined', 'writer should have a ready property');
  assert_equals(typeof writer.ready.then, 'function', 'ready property should be thenable');
  assert_not_equals(typeof writer.closed, 'undefined', 'writer should have a closed property');
  assert_equals(typeof writer.closed.then, 'function', 'closed property should be thenable');
}, 'WritableStream instances should have standard methods and properties');

test(() => {
  ['WritableStreamDefaultWriter', 'WritableStreamDefaultController'].forEach(c =>
      assert_equals(typeof self[c], 'undefined', `${c} should not be exported`));
}, 'private constructors should not be exported');

test(() => {
  let WritableStreamDefaultController;
  new WritableStream({
    start(c) {
      WritableStreamDefaultController = c.constructor;
    }
  });

  assert_throws(new TypeError(), () => new WritableStreamDefaultController({}),
                'constructor should throw a TypeError exception');
}, 'WritableStreamDefaultController constructor should throw unless passed a WritableStream');

test(() => {
  let WritableStreamDefaultController;
  const stream = new WritableStream({
    start(c) {
      WritableStreamDefaultController = c.constructor;
    }
  });

  assert_throws(new TypeError(), () => new WritableStreamDefaultController(stream),
                'constructor should throw a TypeError exception');
}, 'WritableStreamDefaultController constructor should throw when passed an initialised WritableStream');

test(() => {
  const stream = new WritableStream();
  const writer = stream.getWriter();
  const WritableStreamDefaultWriter = writer.constructor;
  writer.releaseLock();
  assert_throws(new TypeError(), () => new WritableStreamDefaultWriter({}),
                'constructor should throw a TypeError exception');
}, 'WritableStreamDefaultWriter should throw unless passed a WritableStream');

test(() => {
  const stream = new WritableStream();
  const writer = stream.getWriter();
  const WritableStreamDefaultWriter = writer.constructor;
  assert_throws(new TypeError(), () => new WritableStreamDefaultWriter(stream),
                'constructor should throw a TypeError exception');
}, 'WritableStreamDefaultWriter constructor should throw when stream argument is locked');

done();
