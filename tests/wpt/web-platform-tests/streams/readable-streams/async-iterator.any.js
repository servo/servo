// META: global=worker,jsshell
// META: script=../resources/rs-utils.js
// META: script=../resources/test-utils.js
// META: script=../resources/recording-streams.js
'use strict';

test(() => {
  assert_equals(ReadableStream.prototype[Symbol.asyncIterator], ReadableStream.prototype.getIterator);
}, '@@asyncIterator() method is === to getIterator() method');

test(() => {
  const s = new ReadableStream();
  const it = s.getIterator();
  const proto = Object.getPrototypeOf(it);

  const AsyncIteratorPrototype = Object.getPrototypeOf(Object.getPrototypeOf(async function* () {}).prototype);
  assert_equals(Object.getPrototypeOf(proto), AsyncIteratorPrototype, 'prototype should extend AsyncIteratorPrototype');

  const methods = ['next', 'return'].sort();
  assert_array_equals(Object.getOwnPropertyNames(proto).sort(), methods, 'should have all the correct methods');

  for (const m of methods) {
    const propDesc = Object.getOwnPropertyDescriptor(proto, m);
    assert_false(propDesc.enumerable, 'method should be non-enumerable');
    assert_true(propDesc.configurable, 'method should be configurable');
    assert_true(propDesc.writable, 'method should be writable');
    assert_equals(typeof it[m], 'function', 'method should be a function');
    assert_equals(it[m].name, m, 'method should have the correct name');
  }

  assert_equals(it.next.length, 0, 'next should have no parameters');
  assert_equals(it.return.length, 1, 'return should have 1 parameter');
  assert_equals(typeof it.throw, 'undefined', 'throw should not exist');
}, 'Async iterator instances should have the correct list of properties');

promise_test(async () => {
  const s = new ReadableStream({
    start(c) {
      c.enqueue(1);
      c.enqueue(2);
      c.enqueue(3);
      c.close();
    },
  });

  const chunks = [];
  for await (const chunk of s) {
    chunks.push(chunk);
  }
  assert_array_equals(chunks, [1, 2, 3]);
}, 'Async-iterating a push source');

promise_test(async () => {
  let i = 1;
  const s = new ReadableStream({
    pull(c) {
      c.enqueue(i);
      if (i >= 3) {
        c.close();
      }
      i += 1;
    },
  });

  const chunks = [];
  for await (const chunk of s) {
    chunks.push(chunk);
  }
  assert_array_equals(chunks, [1, 2, 3]);
}, 'Async-iterating a pull source');

promise_test(async () => {
  let i = 1;
  const s = recordingReadableStream({
    pull(c) {
      c.enqueue(i);
      if (i >= 3) {
        c.close();
      }
      i += 1;
    },
  }, new CountQueuingStrategy({ highWaterMark: 0 }));

  const it = s.getIterator();
  assert_array_equals(s.events, []);

  const read1 = await it.next();
  assert_equals(read1.done, false);
  assert_equals(read1.value, 1);
  assert_array_equals(s.events, ['pull']);

  const read2 = await it.next();
  assert_equals(read2.done, false);
  assert_equals(read2.value, 2);
  assert_array_equals(s.events, ['pull', 'pull']);

  const read3 = await it.next();
  assert_equals(read3.done, false);
  assert_equals(read3.value, 3);
  assert_array_equals(s.events, ['pull', 'pull', 'pull']);

  const read4 = await it.next();
  assert_equals(read4.done, true);
  assert_equals(read4.value, undefined);
  assert_array_equals(s.events, ['pull', 'pull', 'pull']);
}, 'Async-iterating a pull source manually');

promise_test(async () => {
  const s = new ReadableStream({
    start(c) {
      c.error('e');
    },
  });

  try {
    for await (const chunk of s) {}
    assert_unreached();
  } catch (e) {
    assert_equals(e, 'e');
  }
}, 'Async-iterating an errored stream throws');

promise_test(async () => {
  const s = new ReadableStream({
    start(c) {
      c.close();
    }
  });

  for await (const chunk of s) {
    assert_unreached();
  }
}, 'Async-iterating a closed stream never executes the loop body, but works fine');

promise_test(async () => {
  const s = new ReadableStream();

  const loop = async () => {
    for await (const chunk of s) {
      assert_unreached();
    }
    assert_unreached();
  };

  await Promise.race([
    loop(),
    flushAsyncEvents()
  ]);
}, 'Async-iterating an empty but not closed/errored stream never executes the loop body and stalls the async function');

promise_test(async () => {
  const s = new ReadableStream({
    start(c) {
      c.enqueue(1);
      c.enqueue(2);
      c.enqueue(3);
      c.close();
    },
  });

  const reader = s.getReader();
  const readResult = await reader.read();
  assert_equals(readResult.done, false);
  assert_equals(readResult.value, 1);
  reader.releaseLock();

  const chunks = [];
  for await (const chunk of s) {
    chunks.push(chunk);
  }
  assert_array_equals(chunks, [2, 3]);
}, 'Async-iterating a partially consumed stream');

for (const type of ['throw', 'break', 'return']) {
  for (const preventCancel of [false, true]) {
    promise_test(async () => {
      const s = recordingReadableStream({
        start(c) {
          c.enqueue(0);
        }
      });

      // use a separate function for the loop body so return does not stop the test
      const loop = async () => {
        for await (const c of s.getIterator({ preventCancel })) {
          if (type === 'throw') {
            throw new Error();
          } else if (type === 'break') {
            break;
          } else if (type === 'return') {
            return;
          }
        }
      };

      try {
        await loop();
      } catch (e) {}

      if (preventCancel) {
        assert_array_equals(s.events, ['pull'], `cancel() should not be called`);
      } else {
        assert_array_equals(s.events, ['pull', 'cancel', undefined], `cancel() should be called`);
      }
    }, `Cancellation behavior when ${type}ing inside loop body; preventCancel = ${preventCancel}`);
  }
}

for (const preventCancel of [false, true]) {
  promise_test(async () => {
    const s = recordingReadableStream({
      start(c) {
        c.enqueue(0);
      }
    });

    const it = s.getIterator({ preventCancel });
    await it.return();

    if (preventCancel) {
      assert_array_equals(s.events, [], `cancel() should not be called`);
    } else {
      assert_array_equals(s.events, ['cancel', undefined], `cancel() should be called`);
    }
  }, `Cancellation behavior when manually calling return(); preventCancel = ${preventCancel}`);
}

promise_test(async t => {
  const s = new ReadableStream();
  const it = s[Symbol.asyncIterator]();
  await it.return();
  return promise_rejects(t, new TypeError(), it.return(), 'return should reject');
}, 'Calling return() twice rejects');

promise_test(async () => {
  const s = new ReadableStream({
    start(c) {
      c.enqueue(0);
      c.close();
    },
  });
  const it = s[Symbol.asyncIterator]();
  const next = await it.next();
  assert_equals(Object.getPrototypeOf(next), Object.prototype);
  assert_array_equals(Object.getOwnPropertyNames(next).sort(), ['done', 'value']);
}, 'next()\'s fulfillment value has the right shape');

promise_test(async t => {
  const s = recordingReadableStream();
  const it = s[Symbol.asyncIterator]();
  it.next();

  await promise_rejects(t, new TypeError(), it.return(), 'return() should reject');
  assert_array_equals(s.events, ['pull']);
}, 'calling return() while there are pending reads rejects');

test(() => {
  const s = new ReadableStream({
    start(c) {
      c.enqueue(0);
      c.close();
    },
  });
  const it = s.getIterator();
  assert_throws(new TypeError(), () => s.getIterator(), 'getIterator() should throw');
}, 'getIterator() throws if there\'s already a lock');

promise_test(async () => {
  const s = new ReadableStream({
    start(c) {
      c.enqueue(1);
      c.enqueue(2);
      c.enqueue(3);
      c.close();
    },
  });

  const chunks = [];
  for await (const chunk of s) {
    chunks.push(chunk);
  }
  assert_array_equals(chunks, [1, 2, 3]);

  const reader = s.getReader();
  await reader.closed;
}, 'Acquiring a reader after exhaustively async-iterating a stream');

promise_test(async () => {
  const s = new ReadableStream({
    start(c) {
      c.enqueue(1);
      c.enqueue(2);
      c.enqueue(3);
      c.close();
    },
  });

  // read the first two chunks, then cancel
  const chunks = [];
  for await (const chunk of s) {
    chunks.push(chunk);
    if (chunk >= 2) {
      break;
    }
  }
  assert_array_equals(chunks, [1, 2]);

  const reader = s.getReader();
  await reader.closed;
}, 'Acquiring a reader after partially async-iterating a stream');

promise_test(async () => {
  const s = new ReadableStream({
    start(c) {
      c.enqueue(1);
      c.enqueue(2);
      c.enqueue(3);
      c.close();
    },
  });

  // read the first two chunks, then release lock
  const chunks = [];
  for await (const chunk of s.getIterator({preventCancel: true})) {
    chunks.push(chunk);
    if (chunk >= 2) {
      break;
    }
  }
  assert_array_equals(chunks, [1, 2]);

  const reader = s.getReader();
  const readResult = await reader.read();
  assert_equals(readResult.done, false, 'should not be closed yet');
  assert_equals(readResult.value, 3, 'should read remaining chunk');
  await reader.closed;
}, 'Acquiring a reader and reading the remaining chunks after partially async-iterating a stream with preventCancel = true');

promise_test(async t => {
  const rs = new ReadableStream();
  const it = rs.getIterator();
  await it.return();
  return promise_rejects(t, new TypeError(), it.next(), 'next() should reject');
}, 'calling next() after return() should reject');

for (const preventCancel of [false, true]) {
  test(() => {
    const rs = new ReadableStream();
    rs.getIterator({ preventCancel }).return();
    // The test passes if this line doesn't throw.
    rs.getReader();
  }, `return() should unlock the stream synchronously when preventCancel = ${preventCancel}`);
}
