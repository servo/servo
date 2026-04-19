// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/assertThrowsValue.js]
description: |
  pending
esid: pending
---*/
// Test that IteratorClose is called when a Generator is abruptly completed by
// Generator.return.

var returnCalled = 0;
function* wrapNoThrow() {
  let iter = {
    [Symbol.iterator]() {
      return this;
    },
    next() {
      return { value: 10, done: false };
    },
    return() {
      returnCalled++;
      return {};
    }
  };
  for (const i of iter) {
    yield i;
  }
}

// Breaking calls Generator.return, which causes the yield above to resume with
// an abrupt completion of kind "return", which then calls
// iter.return.
for (const i of wrapNoThrow()) {
  break;
}
assert.sameValue(returnCalled, 1);

function* wrapThrow() {
  let iter = {
    [Symbol.iterator]() {
      return this;
    },
    next() {
      return { value: 10, done: false };
    },
    return() {
      throw 42;
    }
  };
  for (const i of iter) {
    yield i;
  }
}

// Breaking calls Generator.return, which, like above, calls iter.return. If
// iter.return throws, since the yield is resuming with an abrupt completion of
// kind "return", the newly thrown value takes precedence over returning.
assertThrowsValue(() => {
  for (const i of wrapThrow()) {
    break;
  }
}, 42);

