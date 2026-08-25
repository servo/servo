// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-aggregate-error
description: >
  Abrupt completions of ToString(Symbol message)
info: |
  AggregateError ( errors, message )

  ...
  5. If message is not undefined, then
    a. Let msg be ? ToString(message).
    b. Perform ! CreateMethodProperty(O, "message", msg).
  6. Return O.
features: [AggregateError, Symbol, Symbol.toPrimitive]
---*/

var case1 = Symbol();

assert.throws(TypeError, () => {
  new AggregateError([], case1);
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
  new AggregateError([], case2);
}, 'from ToPrimitive');
