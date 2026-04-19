// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-aggregate-error
description: >
  Abrupt completions of ToString(message)
info: |
  AggregateError ( errors, message )

  ...
  5. If message is not undefined, then
    a. Let msg be ? ToString(message).
    b. Perform ! CreateMethodProperty(O, "message", msg).
  6. Return O.
features: [AggregateError, Symbol.toPrimitive]
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
  new AggregateError([], case1);
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
  new AggregateError([], case2);
}, 'toString');

var case3 = {
  [Symbol.toPrimitive]: undefined,
  toString: undefined,
  valueOf() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, () => {
  new AggregateError([], case3);
}, 'valueOf');
