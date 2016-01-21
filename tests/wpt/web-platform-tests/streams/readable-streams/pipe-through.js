'use strict';

if (self.importScripts) {
  self.importScripts('../resources/test-utils.js');
  self.importScripts('/resources/testharness.js');
}

test(() => {

  let pipeToArguments;
  const thisValue = {
    pipeTo() {
      pipeToArguments = arguments;
    }
  };

  const input = { readable: {}, writable: {} };
  const options = {};
  const result = ReadableStream.prototype.pipeThrough.call(thisValue, input, options);

  assert_array_equals(pipeToArguments, [input.writable, options],
    'correct arguments should be passed to thisValue.pipeTo');
  assert_equals(result, input.readable, 'return value should be the passed readable property');

}, 'ReadableStream.prototype.pipeThrough should work generically on its this and its arguments');

test(() => {

  const thisValue = {
    pipeTo() {
      assert_unreached('pipeTo should not be called');
    }
  };

  methodThrows(ReadableStream.prototype, 'pipeThrough', thisValue, [undefined, {}]);
  methodThrows(ReadableStream.prototype, 'pipeThrough', thisValue, [null, {}]);

}, 'ReadableStream.prototype.pipeThrough should throw when its first argument is not convertible to an object');

test(() => {

  const args = [{ readable: {}, writable: {} }, {}];

  methodThrows(ReadableStream.prototype, 'pipeThrough', undefined, args);
  methodThrows(ReadableStream.prototype, 'pipeThrough', null, args);
  methodThrows(ReadableStream.prototype, 'pipeThrough', 1, args);
  methodThrows(ReadableStream.prototype, 'pipeThrough', { pipeTo: 'test' }, args);

}, 'ReadableStream.prototype.pipeThrough should throw when "this" has no pipeTo method');

test(() => {
  const error = new Error('potato');

  const throwingPipeTo = {
    get pipeTo() {
      throw error;
    }
  };
  assert_throws(error,
    () => ReadableStream.prototype.pipeThrough.call(throwingPipeTo, { readable: { }, writable: { } }, {}),
    'pipeThrough should rethrow the error thrown by pipeTo');

  const thisValue = {
    pipeTo() {
      assert_unreached('pipeTo should not be called');
    }
  };

  const throwingWritable = {
    readable: {},
    get writable() {
      throw error;
    }
  };
  assert_throws(error,
    () => ReadableStream.prototype.pipeThrough.call(thisValue, throwingWritable, {}),
    'pipeThrough should rethrow the error thrown by the writable getter');

  const throwingReadable = {
    get readable() {
      throw error;
    },
    writable: {}
  };
  assert_throws(error,
    () => ReadableStream.prototype.pipeThrough.call(thisValue, throwingReadable, {}),
    'pipeThrough should rethrow the error thrown by the readable getter');

}, 'ReadableStream.prototype.pipeThrough should rethrow errors from accessing pipeTo, readable, or writable');

test(() => {

  let count = 0;
  const thisValue = {
    pipeTo() {
      ++count;
    }
  };

  ReadableStream.prototype.pipeThrough.call(thisValue, { readable: {}, writable: {} });
  ReadableStream.prototype.pipeThrough.call(thisValue, { readable: {} }, {});
  ReadableStream.prototype.pipeThrough.call(thisValue, { writable: {} }, {});

  assert_equals(count, 3, 'pipeTo was called 3 times');

}, 'ReadableStream.prototype.pipeThrough should work with missing readable, writable, or options');

done();
