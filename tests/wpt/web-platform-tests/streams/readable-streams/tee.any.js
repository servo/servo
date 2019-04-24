// META: global=worker,jsshell
// META: script=../resources/rs-utils.js
// META: script=../resources/test-utils.js
// META: script=../resources/recording-streams.js
'use strict';

test(() => {

  const rs = new ReadableStream();
  const result = rs.tee();

  assert_true(Array.isArray(result), 'return value should be an array');
  assert_equals(result.length, 2, 'array should have length 2');
  assert_equals(result[0].constructor, ReadableStream, '0th element should be a ReadableStream');
  assert_equals(result[1].constructor, ReadableStream, '1st element should be a ReadableStream');

}, 'ReadableStream teeing: rs.tee() returns an array of two ReadableStreams');

promise_test(t => {

  const rs = new ReadableStream({
    start(c) {
      c.enqueue('a');
      c.enqueue('b');
      c.close();
    }
  });

  const branch = rs.tee();
  const branch1 = branch[0];
  const branch2 = branch[1];
  const reader1 = branch1.getReader();
  const reader2 = branch2.getReader();

  reader2.closed.then(t.unreached_func('branch2 should not be closed'));

  return Promise.all([
    reader1.closed,
    reader1.read().then(r => {
      assert_object_equals(r, { value: 'a', done: false }, 'first chunk from branch1 should be correct');
    }),
    reader1.read().then(r => {
      assert_object_equals(r, { value: 'b', done: false }, 'second chunk from branch1 should be correct');
    }),
    reader1.read().then(r => {
      assert_object_equals(r, { value: undefined, done: true }, 'third read() from branch1 should be done');
    }),
    reader2.read().then(r => {
      assert_object_equals(r, { value: 'a', done: false }, 'first chunk from branch2 should be correct');
    })
  ]);

}, 'ReadableStream teeing: should be able to read one branch to the end without affecting the other');

promise_test(() => {

  const theObject = { the: 'test object' };
  const rs = new ReadableStream({
    start(c) {
      c.enqueue(theObject);
    }
  });

  const branch = rs.tee();
  const branch1 = branch[0];
  const branch2 = branch[1];
  const reader1 = branch1.getReader();
  const reader2 = branch2.getReader();

  return Promise.all([reader1.read(), reader2.read()]).then(values => {
    assert_object_equals(values[0], values[1], 'the values should be equal');
  });

}, 'ReadableStream teeing: values should be equal across each branch');

promise_test(t => {

  const theError = { name: 'boo!' };
  const rs = new ReadableStream({
    start(c) {
      c.enqueue('a');
      c.enqueue('b');
    },
    pull() {
      throw theError;
    }
  });

  const branches = rs.tee();
  const reader1 = branches[0].getReader();
  const reader2 = branches[1].getReader();

  reader1.label = 'reader1';
  reader2.label = 'reader2';

  return Promise.all([
    promise_rejects(t, theError, reader1.closed),
    promise_rejects(t, theError, reader2.closed),
    reader1.read().then(r => {
      assert_object_equals(r, { value: 'a', done: false }, 'should be able to read the first chunk in branch1');
    }),
    reader1.read().then(r => {
      assert_object_equals(r, { value: 'b', done: false }, 'should be able to read the second chunk in branch1');

      return promise_rejects(t, theError, reader2.read());
    })
    .then(() => promise_rejects(t, theError, reader1.read()))
  ]);

}, 'ReadableStream teeing: errors in the source should propagate to both branches');

promise_test(() => {

  const rs = new ReadableStream({
    start(c) {
      c.enqueue('a');
      c.enqueue('b');
      c.close();
    }
  });

  const branches = rs.tee();
  const branch1 = branches[0];
  const branch2 = branches[1];
  branch1.cancel();

  return Promise.all([
    readableStreamToArray(branch1).then(chunks => {
      assert_array_equals(chunks, [], 'branch1 should have no chunks');
    }),
    readableStreamToArray(branch2).then(chunks => {
      assert_array_equals(chunks, ['a', 'b'], 'branch2 should have two chunks');
    })
  ]);

}, 'ReadableStream teeing: canceling branch1 should not impact branch2');

promise_test(() => {

  const rs = new ReadableStream({
    start(c) {
      c.enqueue('a');
      c.enqueue('b');
      c.close();
    }
  });

  const branches = rs.tee();
  const branch1 = branches[0];
  const branch2 = branches[1];
  branch2.cancel();

  return Promise.all([
    readableStreamToArray(branch1).then(chunks => {
      assert_array_equals(chunks, ['a', 'b'], 'branch1 should have two chunks');
    }),
    readableStreamToArray(branch2).then(chunks => {
      assert_array_equals(chunks, [], 'branch2 should have no chunks');
    })
  ]);

}, 'ReadableStream teeing: canceling branch2 should not impact branch1');

promise_test(() => {

  const reason1 = new Error('We\'re wanted men.');
  const reason2 = new Error('I have the death sentence on twelve systems.');

  let resolve;
  const promise = new Promise(r => resolve = r);
  const rs = new ReadableStream({
    cancel(reason) {
      assert_array_equals(reason, [reason1, reason2],
                          'the cancel reason should be an array containing those from the branches');
      resolve();
    }
  });

  const branch = rs.tee();
  const branch1 = branch[0];
  const branch2 = branch[1];
  branch1.cancel(reason1);
  branch2.cancel(reason2);

  return promise;

}, 'ReadableStream teeing: canceling both branches should aggregate the cancel reasons into an array');

promise_test(() => {

  const reason1 = new Error('This little one\'s not worth the effort.');
  const reason2 = new Error('Come, let me get you something.');

  let resolve;
  const promise = new Promise(r => resolve = r);
  const rs = new ReadableStream({
    cancel(reason) {
      assert_array_equals(reason, [reason1, reason2],
                          'the cancel reason should be an array containing those from the branches');
      resolve();
    }
  });

  const branch = rs.tee();
  const branch1 = branch[0];
  const branch2 = branch[1];
  return Promise.all([
    branch2.cancel(reason2),
    branch1.cancel(reason1),
    promise
  ]);

}, 'ReadableStream teeing: canceling both branches in reverse order should aggregate the cancel reasons into an array');

promise_test(t => {

  const theError = { name: 'I\'ll be careful.' };
  const rs = new ReadableStream({
    cancel() {
      throw theError;
    }
  });

  const branch = rs.tee();
  const branch1 = branch[0];
  const branch2 = branch[1];

  return Promise.all([
    promise_rejects(t, theError, branch1.cancel()),
    promise_rejects(t, theError, branch2.cancel())
  ]);

}, 'ReadableStream teeing: failing to cancel the original stream should cause cancel() to reject on branches');

test(() => {

  let controller;
  const stream = new ReadableStream({ start(c) { controller = c; } });
  const [branch1, branch2] = stream.tee();

  controller.error("error");

  branch1.cancel().catch(_=>_);
  branch2.cancel().catch(_=>_);

}, 'ReadableStream teeing: erroring a teed stream should properly handle canceled branches');

promise_test(t => {

  let controller;
  const stream = new ReadableStream({ start(c) { controller = c; } });
  const [branch1, branch2] = stream.tee();

  const error = new Error();
  error.name = 'distinctive';

  // Ensure neither branch is waiting in ReadableStreamDefaultReaderRead().
  controller.enqueue();
  controller.enqueue();

  return delay(0).then(() => {
    // This error will have to be detected via [[closedPromise]].
    controller.error(error);

    const reader1 = branch1.getReader();
    const reader2 = branch2.getReader();

    return Promise.all([
      promise_rejects(t, error, reader1.closed, 'reader1.closed should reject'),
      promise_rejects(t, error, reader2.closed, 'reader2.closed should reject')
    ]);
  });

}, 'ReadableStream teeing: erroring a teed stream should error both branches');

promise_test(() => {

  let controller;
  const rs = new ReadableStream({
    start(c) {
      controller = c;
    }
  });

  const branches = rs.tee();
  const reader1 = branches[0].getReader();
  const reader2 = branches[1].getReader();

  const promise = Promise.all([reader1.closed, reader2.closed]);

  controller.close();
  return promise;

}, 'ReadableStream teeing: closing the original should immediately close the branches');

promise_test(t => {

  let controller;
  const rs = new ReadableStream({
    start(c) {
      controller = c;
    }
  });

  const branches = rs.tee();
  const reader1 = branches[0].getReader();
  const reader2 = branches[1].getReader();

  const theError = { name: 'boo!' };
  const promise = Promise.all([
    promise_rejects(t, theError, reader1.closed),
    promise_rejects(t, theError, reader2.closed)
  ]);

  controller.error(theError);
  return promise;

}, 'ReadableStream teeing: erroring the original should immediately error the branches');

test(t => {

  // Copy original global.
  const oldReadableStream = ReadableStream;
  const getReader = ReadableStream.prototype.getReader;

  const origRS = new ReadableStream();

  // Replace the global ReadableStream constructor with one that doesn't work.
  ReadableStream = function() {
    throw new Error('global ReadableStream constructor called');
  };
  t.add_cleanup(() => {
    ReadableStream = oldReadableStream;
  });

  // This will probably fail if the global ReadableStream constructor was used.
  const [rs1, rs2] = origRS.tee();

  // These will definitely fail if the global ReadableStream constructor was used.
  assert_not_equals(getReader.call(rs1), undefined, 'getReader should work on rs1');
  assert_not_equals(getReader.call(rs2), undefined, 'getReader should work on rs2');

}, 'ReadableStreamTee should not use a modified ReadableStream constructor from the global object');

promise_test(t => {

  const rs = recordingReadableStream({}, { highWaterMark: 0 });

  // Create two branches, each with a HWM of 1. This should result in one
  // chunk being pulled, not two.
  rs.tee();
  return flushAsyncEvents().then(() => {
    assert_array_equals(rs.events, ['pull'], 'pull should only be called once');
  });

}, 'ReadableStreamTee should not pull more chunks than can fit in the branch queue');

promise_test(t => {

  const rs = recordingReadableStream({
    pull(controller) {
      controller.enqueue('a');
    }
  }, { highWaterMark: 0 });

  const [reader1, reader2] = rs.tee().map(branch => branch.getReader());
  return Promise.all([reader1.read(), reader2.read()])
      .then(() => {
    assert_array_equals(rs.events, ['pull', 'pull'], 'pull should be called twice');
  });

}, 'ReadableStreamTee should only pull enough to fill the emptiest queue');

promise_test(t => {

  const rs = recordingReadableStream({}, { highWaterMark: 0 });
  const theError = { name: 'boo!' };

  rs.controller.error(theError);

  const [reader1, reader2] = rs.tee().map(branch => branch.getReader());

  return flushAsyncEvents().then(() => {
    assert_array_equals(rs.events, [], 'pull should not be called');

    return Promise.all([
      promise_rejects(t, theError, reader1.closed),
      promise_rejects(t, theError, reader2.closed)
    ]);
  });

}, 'ReadableStreamTee should not pull when original is already errored');

for (const branch of [1, 2]) {
  promise_test(t => {

    const rs = recordingReadableStream({}, { highWaterMark: 0 });
    const theError = { name: 'boo!' };

    const [reader1, reader2] = rs.tee().map(branch => branch.getReader());

    return flushAsyncEvents().then(() => {
      assert_array_equals(rs.events, ['pull'], 'pull should be called once');

      rs.controller.enqueue('a');

      const reader = (branch === 1) ? reader1 : reader2;
      return reader.read();
    }).then(() => flushAsyncEvents()).then(() => {
      assert_array_equals(rs.events, ['pull', 'pull'], 'pull should be called twice');

      rs.controller.error(theError);

      return Promise.all([
        promise_rejects(t, theError, reader1.closed),
        promise_rejects(t, theError, reader2.closed)
      ]);
    }).then(() => flushAsyncEvents()).then(() => {
      assert_array_equals(rs.events, ['pull', 'pull'], 'pull should be called twice');
    });

  }, `ReadableStreamTee stops pulling when original stream errors while branch ${branch} is reading`);
}

promise_test(t => {

  const rs = recordingReadableStream({}, { highWaterMark: 0 });
  const theError = { name: 'boo!' };

  const [reader1, reader2] = rs.tee().map(branch => branch.getReader());

  return flushAsyncEvents().then(() => {
    assert_array_equals(rs.events, ['pull'], 'pull should be called once');

    rs.controller.enqueue('a');

    return Promise.all([reader1.read(), reader2.read()]);
  }).then(() => flushAsyncEvents()).then(() => {
    assert_array_equals(rs.events, ['pull', 'pull'], 'pull should be called twice');

    rs.controller.error(theError);

    return Promise.all([
      promise_rejects(t, theError, reader1.closed),
      promise_rejects(t, theError, reader2.closed)
    ]);
  }).then(() => flushAsyncEvents()).then(() => {
    assert_array_equals(rs.events, ['pull', 'pull'], 'pull should be called twice');
  });

}, 'ReadableStreamTee stops pulling when original stream errors while both branches are reading');
