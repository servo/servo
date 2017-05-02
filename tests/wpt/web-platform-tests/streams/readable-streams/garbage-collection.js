'use strict';

if (self.importScripts) {
  self.importScripts('../resources/test-utils.js');
  self.importScripts('/resources/testharness.js');
}

promise_test(() => {

  let controller;
  new ReadableStream({
    start(c) {
      controller = c;
    }
  });

  garbageCollect();

  return delay(50).then(() => {
    controller.close();
    assert_throws(new TypeError(), () => controller.close(), 'close should throw a TypeError the second time');
    assert_throws(new TypeError(), () => controller.error(), 'error should throw a TypeError on a closed stream');
  });

}, 'ReadableStreamController methods should continue working properly when scripts lose their reference to the ' +
   'readable stream');

promise_test(() => {

  let controller;

  const closedPromise = new ReadableStream({
    start(c) {
      controller = c;
    }
  }).getReader().closed;

  garbageCollect();

  return delay(50).then(() => controller.close()).then(() => closedPromise);

}, 'ReadableStream closed promise should fulfill even if the stream and reader JS references are lost');

promise_test(t => {

  const theError = new Error('boo');
  let controller;

  const closedPromise = new ReadableStream({
    start(c) {
      controller = c;
    }
  }).getReader().closed;

  garbageCollect();

  return delay(50).then(() => controller.error(theError))
                  .then(() => promise_rejects(t, theError, closedPromise));

}, 'ReadableStream closed promise should reject even if stream and reader JS references are lost');

promise_test(() => {

  const rs = new ReadableStream({});

  rs.getReader();

  garbageCollect();

  return delay(50).then(() => assert_throws(new TypeError(), () => rs.getReader(),
    'old reader should still be locking the stream even after garbage collection'));

}, 'Garbage-collecting a ReadableStreamDefaultReader should not unlock its stream');

done();
