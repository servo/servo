// The concurrency tests assert that no pair of asynchronous I/O operations is
// run concurrently. For any operation A, there is one file containing the tests
// asserting that while A is running, any other operation B rejects. In order to
// avoid constructing a quadratic number of tests, the rejecting operations are
// abstracted here.
//
// For any rejecting operation B, define the following.
//   name: The operation's name
//   prepare: Code needed before performing the operation, e.g., buffer
//              allocation. Returns an object to be passed to assertRejection
//              and assertUnchanged. May be empty.
//   assertRejection: A promise_rejects_dom(...) statement that calls
//                      operation B. Takes the object returned by prepare as
//                      third parameter. Returns a promise.
//   assertUnchanged: An assertion that rejecting the promise did not change
//                      the buffers in unexpected ways. The assertion does not
//                      check if the file itself was not changed. This will be
//                      after performing operation A in its file. Takes the
//                      object returned by prepare as parameter. May be empty.
//
// The array kOperations contains all abstractions.

const kOperations = [];

(() => {
  const kOpRead = {
    name: 'read',
    prepare: () => {
      const readBuffer = new Uint8Array(4);
      return readBuffer;
    },
    assertRejection: async (testCase, file, readBuffer) => {
      await promise_rejects_dom(testCase, 'InvalidStateError',
                                 file.read(readBuffer, 4));
    },
    assertUnchanged: (readBuffer) => {
      assert_array_equals(readBuffer, [0, 0, 0, 0]);
    },
  };
  kOperations.push(kOpRead);

  const kOpWrite = {
    name: 'write',
    prepare: () => {
      const writeBuffer = new Uint8Array(4);
      writeBuffer.set([96, 97, 98, 99]);
      return writeBuffer;
    },
    assertRejection: async (testCase, file, writeBuffer) => {
      await promise_rejects_dom(testCase, 'InvalidStateError',
                                 file.write(writeBuffer, 4));
    },
    assertUnchanged: () => {},
  };
  kOperations.push(kOpWrite);

  const kOpGetLength = {
    name: 'getLength',
    prepare: () => {},
    assertRejection: async (testCase, file) => {
      await promise_rejects_dom(testCase, 'InvalidStateError',
                                 file.getLength());
    },
    assertUnchanged: () => {},
  };
  kOperations.push(kOpGetLength);

  const kOpFlush = {
    name: 'flush',
    prepare: () => {},
    assertRejection: async (testCase, file) => {
      await promise_rejects_dom(testCase, 'InvalidStateError',
                                file.flush());
    },
    assertUnchanged: () => {},
  };
  kOperations.push(kOpFlush);

  const kOpSetLength = {
    name: 'setLength',
    prepare: () => {},
    assertRejection: async (testCase, file) => {
      await promise_rejects_dom(testCase, 'InvalidStateError',
                                file.setLength(2));
    },
    assertUnchanged: () => {},
  };
  kOperations.push(kOpSetLength);
})();
