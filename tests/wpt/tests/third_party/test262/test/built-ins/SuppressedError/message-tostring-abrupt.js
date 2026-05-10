// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-suppressederror-constructor
description: >
  Abrupt completions of ToString(message)
info: |
  SuppressedError ( error, suppressed, message )

  ...
  5. If message is not undefined, then
    a. Let msg be ? ToString(message).
    b. Perform ! CreateMethodProperty(O, "message", msg).
  6. Return O.
features: [explicit-resource-management, Symbol.toPrimitive]
---*/

var case1 = {
  [Symbol.toPrimitive]() {
    throw new Test262Error();
  },
  toString() {
    throw 'toString called';
  },
  valueOf() {
    throw 'valueOf called';
  }
};

assert.throws(Test262Error, () => {
  new SuppressedError(undefined, undefined, case1);
}, 'toPrimitive');

var case2 = {
  [Symbol.toPrimitive]: undefined,
  toString() {
    throw new Test262Error();
  },
  valueOf() {
    throw 'valueOf called';
  }
};

assert.throws(Test262Error, () => {
  new SuppressedError(undefined, undefined, case2);
}, 'toString');

var case3 = {
  [Symbol.toPrimitive]: undefined,
  toString: undefined,
  valueOf() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, () => {
  new SuppressedError(undefined, undefined, case3);
}, 'valueOf');
