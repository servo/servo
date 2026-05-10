// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
features:
  - iterator-helpers
description: |
  pending
esid: pending
---*/

class TestError extends Error {}

function checkIterResult({done, value}, expectedDone, expectedValue) {
  assert.sameValue(done, expectedDone);
  assert.sameValue(value, expectedValue);
}

const iter = {
  next(value) {
    return {done: false, value: arguments.length};
  },
  return() {
    throw new TestError();
  },
  throw: (value) => ({done: true, value}),
};
const thisWrap = Iterator.from(iter);
const otherGlobal = $262.createRealm().global;
const otherWrap = otherGlobal.Iterator.from(iter);

checkIterResult(thisWrap.next.call(otherWrap), false, 0);
checkIterResult(thisWrap.next.call(otherWrap, 'value'), false, 0);

assert.throws(TestError, thisWrap.return.bind(otherWrap));

