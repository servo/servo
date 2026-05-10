// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-suppressederror-constructor
description: >
  Abrupt completions of ToString(Symbol message)
info: |
  SuppressedError ( error, suppressed, message )

  ...
  5. If message is not undefined, then
    a. Let msg be ? ToString(message).
    b. Perform ! CreateMethodProperty(O, "message", msg).
  6. Return O.
features: [explicit-resource-management, Symbol, Symbol.toPrimitive]
---*/

var case1 = Symbol();

assert.throws(TypeError, () => {
  new SuppressedError(undefined, undefined, case1);
}, 'toPrimitive');

var case2 = {
  [Symbol.toPrimitive]() {
    return Symbol();
  },
  toString() {
    throw new Test262Error();
  },
  valueOf() {
    throw new Test262Error();
  }
};

assert.throws(TypeError, () => {
  new SuppressedError(undefined, undefined, case2);
}, 'from ToPrimitive');
