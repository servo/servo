// META: global=window,worker,jsshell
// META: script=../resources/test-utils.js
'use strict';

let ReadableStreamDefaultReader;
let ReadableStreamDefaultController;
let ReadableStreamAsyncIteratorPrototype;

test(() => {

  // It's not exposed globally, but we test a few of its properties here.
  ReadableStreamDefaultReader = (new ReadableStream()).getReader().constructor;

}, 'Can get the ReadableStreamDefaultReader constructor indirectly');

test(() => {

  // It's not exposed globally, but we test a few of its properties here.
  new ReadableStream({
    start(c) {
      ReadableStreamDefaultController = c.constructor;
    }
  });

}, 'Can get the ReadableStreamDefaultController constructor indirectly');

test(() => {

  const rs = new ReadableStream();
  ReadableStreamAsyncIteratorPrototype = Object.getPrototypeOf(rs.getIterator());

}, 'Can get ReadableStreamAsyncIteratorPrototype object indirectly');

function fakeRS() {
  return Object.setPrototypeOf({
    cancel() { return Promise.resolve(); },
    getReader() { return new ReadableStreamDefaultReader(new ReadableStream()); },
    pipeThrough(obj) { return obj.readable; },
    pipeTo() { return Promise.resolve(); },
    tee() { return [realRS(), realRS()]; }
  }, ReadableStream.prototype);
}

function realRS() {
  return new ReadableStream();
}

function fakeRSDefaultReader() {
  return Object.setPrototypeOf({
    get closed() { return Promise.resolve(); },
    cancel() { return Promise.resolve(); },
    read() { return Promise.resolve({ value: undefined, done: true }); },
    releaseLock() { return; }
  }, ReadableStreamDefaultReader.prototype);
}

function realRSDefaultReader() {
  return new ReadableStream().getReader();
}

function fakeRSDefaultController() {
  return Object.setPrototypeOf({
    close() { },
    enqueue() { },
    error() { }
  }, ReadableStreamDefaultController.prototype);
}

function realRSDefaultController() {
  let controller;
  new ReadableStream({
    start(c) {
      controller = c;
    }
  });
  return controller;
}

function fakeRSAsyncIterator() {
  return Object.setPrototypeOf({
    next() { },
    return(value = undefined) { }
  }, ReadableStreamAsyncIteratorPrototype);
}

promise_test(t => {

  return methodRejectsForAll(t, ReadableStream.prototype, 'cancel',
                             [fakeRS(), realRSDefaultReader(), realRSDefaultController(), undefined, null]);

}, 'ReadableStream.prototype.cancel enforces a brand check');

test(() => {

  methodThrowsForAll(ReadableStream.prototype, 'getIterator',
                     [fakeRS(), realRSDefaultReader(), realRSDefaultController(), undefined, null]);

}, 'ReadableStream.prototype.getIterator enforces a brand check');

test(() => {

  methodThrowsForAll(ReadableStream.prototype, 'getReader',
                     [fakeRS(), realRSDefaultReader(), realRSDefaultController(), undefined, null]);

}, 'ReadableStream.prototype.getReader enforces a brand check');

test(() => {

  getterThrowsForAll(ReadableStream.prototype, 'locked',
                     [fakeRS(), realRSDefaultReader(), realRSDefaultController(), undefined, null]);

}, 'ReadableStream.prototype.locked enforces a brand check');

test(() => {

  methodThrowsForAll(ReadableStream.prototype, 'tee', [fakeRS(), realRSDefaultReader(), realRSDefaultController(), undefined, null]);

}, 'ReadableStream.prototype.tee enforces a brand check');

test(() => {

  assert_throws_js(TypeError, () => new ReadableStreamDefaultReader(fakeRS()),
                   'Constructing a ReadableStreamDefaultReader should throw');

}, 'ReadableStreamDefaultReader enforces a brand check on its argument');

promise_test(t => {

  return getterRejectsForAll(t, ReadableStreamDefaultReader.prototype, 'closed',
                             [fakeRSDefaultReader(), realRS(), realRSDefaultController(), undefined, null]);

}, 'ReadableStreamDefaultReader.prototype.closed enforces a brand check');

promise_test(t => {

  return methodRejectsForAll(t, ReadableStreamDefaultReader.prototype, 'cancel',
                             [fakeRSDefaultReader(), realRS(), realRSDefaultController(), undefined, null]);

}, 'ReadableStreamDefaultReader.prototype.cancel enforces a brand check');

promise_test(t => {

  return methodRejectsForAll(t, ReadableStreamDefaultReader.prototype, 'read',
                             [fakeRSDefaultReader(), realRS(), realRSDefaultController(), undefined, null]);

}, 'ReadableStreamDefaultReader.prototype.read enforces a brand check');

test(() => {

  methodThrowsForAll(ReadableStreamDefaultReader.prototype, 'releaseLock',
                     [fakeRSDefaultReader(), realRS(), realRSDefaultController(), undefined, null]);

}, 'ReadableStreamDefaultReader.prototype.releaseLock enforces a brand check');

test(() => {

  assert_throws_js(TypeError, () => new ReadableStreamDefaultController(fakeRS()),
                   'Constructing a ReadableStreamDefaultController should throw');

}, 'ReadableStreamDefaultController enforces a brand check on its argument');

test(() => {

  assert_throws_js(TypeError, () => new ReadableStreamDefaultController(realRS()),
                   'Constructing a ReadableStreamDefaultController should throw');

}, 'ReadableStreamDefaultController can\'t be given a fully-constructed ReadableStream');

test(() => {

  getterThrowsForAll(ReadableStreamDefaultController.prototype, 'desiredSize',
                     [fakeRSDefaultController(), realRS(), realRSDefaultReader(), undefined, null]);

}, 'ReadableStreamDefaultController.prototype.desiredSize enforces a brand check');

test(() => {

  methodThrowsForAll(ReadableStreamDefaultController.prototype, 'close',
                     [fakeRSDefaultController(), realRS(), realRSDefaultReader(), undefined, null]);

}, 'ReadableStreamDefaultController.prototype.close enforces a brand check');

test(() => {

  methodThrowsForAll(ReadableStreamDefaultController.prototype, 'enqueue',
                     [fakeRSDefaultController(), realRS(), realRSDefaultReader(), undefined, null]);

}, 'ReadableStreamDefaultController.prototype.enqueue enforces a brand check');

test(() => {

  methodThrowsForAll(ReadableStreamDefaultController.prototype, 'error',
                     [fakeRSDefaultController(), realRS(), realRSDefaultReader(), undefined, null]);

}, 'ReadableStreamDefaultController.prototype.error enforces a brand check');

promise_test(t => {

  return methodRejectsForAll(t, ReadableStreamAsyncIteratorPrototype, 'next',
                             [fakeRSAsyncIterator(), realRS(), realRSDefaultReader(), undefined, null]);

}, 'ReadableStreamAsyncIteratorPrototype.next enforces a brand check');

promise_test(t => {

  return methodRejectsForAll(t, ReadableStreamAsyncIteratorPrototype, 'return',
                             [fakeRSAsyncIterator(), realRS(), realRSDefaultReader(), undefined, null]);

}, 'ReadableStreamAsyncIteratorPrototype.return enforces a brand check');
