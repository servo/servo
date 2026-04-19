// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-aggregate-error
description: >
  Process arguments in superclass-then-subclass order
info: |
  AggregateError ( errors, message )

  TODO: get updated prose

features: [AggregateError, Symbol.iterator]
includes: [promiseHelper.js]
---*/

let sequence = [];
const message = {
  toString() {
    sequence.push(1);
    return '';
  }
};
const errors = {
  [Symbol.iterator]() {
    sequence.push(2);
    return {
      next() {
        sequence.push(3);
        return {
          done: true
        };
      }
    };
  }
};

new AggregateError(errors, message);

assert.sameValue(sequence.length, 3);
checkSequence(sequence);
