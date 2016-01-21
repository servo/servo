'use strict';

if (self.importScripts) {
  self.importScripts('../resources/test-utils.js');
  self.importScripts('/resources/testharness.js');
}

let ReadableStreamReader;
let ReadableStreamController;

test(() => {

  // It's not exposed globally, but we test a few of its properties here.
  ReadableStreamReader = (new ReadableStream()).getReader().constructor;

}, 'Can get the ReadableStreamReader constructor indirectly');

test(() => {

  // It's not exposed globally, but we test a few of its properties here.
  new ReadableStream({
    start(c) {
      ReadableStreamController = c.constructor;
    }
  });

}, 'Can get the ReadableStreamController constructor indirectly');

function fakeReadableStream() {
  return {
    cancel() { return Promise.resolve(); },
    getReader() { return new ReadableStreamReader(new ReadableStream()); },
    pipeThrough(obj) { return obj.readable; },
    pipeTo() { return Promise.resolve(); },
    tee() { return [realReadableStream(), realReadableStream()]; }
  };
}

function realReadableStream() {
  return new ReadableStream();
}

function fakeReadableStreamReader() {
  return {
    get closed() { return Promise.resolve(); },
    cancel() { return Promise.resolve(); },
    read() { return Promise.resolve({ value: undefined, done: true }); },
    releaseLock() { return; }
  };
}

function fakeReadableStreamController() {
  return {
    close() { },
    enqueue() { },
    error() { }
  };
}

promise_test(t => {

  return methodRejects(t, ReadableStream.prototype, 'cancel', fakeReadableStream());

}, 'ReadableStream.prototype.cancel enforces a brand check');

test(() => {

  methodThrows(ReadableStream.prototype, 'getReader', fakeReadableStream());

}, 'ReadableStream.prototype.getReader enforces a brand check');

test(() => {

  methodThrows(ReadableStream.prototype, 'tee', fakeReadableStream());

}, 'ReadableStream.prototype.tee enforces a brand check');

test(() => {

  assert_throws(new TypeError(), () => new ReadableStreamReader(fakeReadableStream()),
                'Constructing a ReadableStreamReader should throw');

}, 'ReadableStreamReader enforces a brand check on its argument');

promise_test(t => {

  return Promise.all([
    getterRejects(t, ReadableStreamReader.prototype, 'closed', fakeReadableStreamReader()),
    getterRejects(t, ReadableStreamReader.prototype, 'closed', realReadableStream())
  ]);

}, 'ReadableStreamReader.prototype.closed enforces a brand check');

promise_test(t => {

  return Promise.all([
    methodRejects(t, ReadableStreamReader.prototype, 'cancel', fakeReadableStreamReader()),
    methodRejects(t, ReadableStreamReader.prototype, 'cancel', realReadableStream())
  ]);

}, 'ReadableStreamReader.prototype.cancel enforces a brand check');

promise_test(t => {

  return Promise.all([
    methodRejects(t, ReadableStreamReader.prototype, 'read', fakeReadableStreamReader()),
    methodRejects(t, ReadableStreamReader.prototype, 'read', realReadableStream())
  ]);

}, 'ReadableStreamReader.prototype.read enforces a brand check');

test(() => {

  methodThrows(ReadableStreamReader.prototype, 'releaseLock', fakeReadableStreamReader());
  methodThrows(ReadableStreamReader.prototype, 'releaseLock', realReadableStream());

}, 'ReadableStreamReader.prototype.releaseLock enforces a brand check');

test(() => {

  assert_throws(new TypeError(), () => new ReadableStreamController(fakeReadableStream()),
                'Constructing a ReadableStreamController should throw');

}, 'ReadableStreamController enforces a brand check on its argument');

test(() => {

  assert_throws(new TypeError(), () => new ReadableStreamController(realReadableStream()),
                'Constructing a ReadableStreamController should throw');

}, 'ReadableStreamController can\'t be given a fully-constructed ReadableStream');

test(() => {

  methodThrows(ReadableStreamController.prototype, 'close', fakeReadableStreamController());

}, 'ReadableStreamController.prototype.close enforces a brand check');

test(() => {

  methodThrows(ReadableStreamController.prototype, 'enqueue', fakeReadableStreamController());

}, 'ReadableStreamController.prototype.enqueue enforces a brand check');

test(() => {

  methodThrows(ReadableStreamController.prototype, 'error', fakeReadableStreamController());

}, 'ReadableStreamController.prototype.error enforces a brand check');

done();
