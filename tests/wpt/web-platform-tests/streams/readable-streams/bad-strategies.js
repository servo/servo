'use strict';

if (self.importScripts) {
  self.importScripts('/resources/testharness.js');
}

test(() => {

  const theError = new Error('a unique string');

  assert_throws(theError, () => {
    new ReadableStream({}, {
      get size() {
        throw theError;
      },
      highWaterMark: 5
    });
  }, 'construction should re-throw the error');

}, 'Readable stream: throwing strategy.size getter');

test(() => {

  const theError = new Error('a unique string');

  let controller;
  const rs = new ReadableStream(
    {
      start(c) {
        controller = c;
      }
    },
    {
      size() {
        controller.error(theError);
        throw theError;
      },
      highWaterMark: 5
    }
  );

  assert_throws(theError, () => {
    controller.enqueue('a');
  }, 'enqueue should re-throw the error');

}, 'Readable stream: strategy.size errors the stream and then throws');

test(() => {

  const theError = new Error('a unique string');

  let controller;
  const rs = new ReadableStream(
    {
      start(c) {
        controller = c;
      }
    },
    {
      size() {
        controller.error(theError);
        return Infinity;
      },
      highWaterMark: 5
    }
  );

  try {
    controller.enqueue('a');
  } catch (error) {
    assert_equals(error.name, 'RangeError', 'enqueue should throw a RangeError');
  }

}, 'Readable stream: strategy.size errors the stream and then returns Infinity');

promise_test(() => {

  const theError = new Error('a unique string');
  const rs = new ReadableStream(
    {
      start(c) {
        assert_throws(theError, () => c.enqueue('a'), 'enqueue should throw the error');
      }
    },
    {
      size() {
        throw theError;
      },
      highWaterMark: 5
    }
  );

  return rs.getReader().closed.catch(e => {
    assert_equals(e, theError, 'closed should reject with the error');
  });

}, 'Readable stream: throwing strategy.size method');

test(() => {

  const theError = new Error('a unique string');

  assert_throws(theError, () => {
    new ReadableStream({}, {
      size() {
        return 1;
      },
      get highWaterMark() {
        throw theError;
      }
    });
  }, 'construction should re-throw the error');

}, 'Readable stream: throwing strategy.highWaterMark getter');

test(() => {

  for (let highWaterMark of [-1, -Infinity]) {
    assert_throws(new RangeError(), () => {
      new ReadableStream({}, {
        size() {
          return 1;
        },
        highWaterMark
      });
    }, 'construction should throw a RangeError for ' + highWaterMark);
  }

  for (let highWaterMark of [NaN, 'foo', {}]) {
    assert_throws(new TypeError(), () => {
      new ReadableStream({}, {
        size() {
          return 1;
        },
        highWaterMark
      });
    }, 'construction should throw a TypeError for ' + highWaterMark);
  }

}, 'Readable stream: invalid strategy.highWaterMark');

promise_test(() => {

  const promises = [];
  for (let size of [NaN, -Infinity, Infinity, -1]) {
    let theError;
    const rs = new ReadableStream(
      {
        start(c) {
          try {
            c.enqueue('hi');
            assert_unreached('enqueue didn\'t throw');
          } catch (error) {
            assert_equals(error.name, 'RangeError', 'enqueue should throw a RangeError for ' + size);
            theError = error;
          }
        }
      },
      {
        size() {
          return size;
        },
        highWaterMark: 5
      }
    );

    promises.push(rs.getReader().closed.catch(e => {
      assert_equals(e, theError, 'closed should reject with the error for ' + size);
    }));
  }

  return Promise.all(promises);

}, 'Readable stream: invalid strategy.size return value');

done();
