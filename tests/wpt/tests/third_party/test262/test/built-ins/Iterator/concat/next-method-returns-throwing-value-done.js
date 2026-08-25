// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Underlying iterator next returns object with throwing value getter, but is already done
info: |
  Iterator.concat ( ...items )

  ...
  3. Let closure be a new Abstract Closure with no parameters that captures iterables and performs the following steps when called:
    a. For each Record iterable of iterables, do
      ...
      v. Repeat, while innerAlive is true,
        1. Let innerValue be ? IteratorStepValue(iteratorRecord).
        ...
features: [iterator-sequencing]
---*/

class ReturnCalledError extends Error {}
class ValueGetterError extends Error {}

let throwingIterator = {
  next() {
    return {
      get value() {
        throw new ValueGetterError();
      },
      done: true,
    };
  },
  return() {
    throw new ReturnCalledError();
  }
};

let iterable = {
  [Symbol.iterator]() {
    return throwingIterator;
  }
};

let iterator = Iterator.concat(iterable);

let iterResult = iterator.next();

assert.sameValue(iterResult.done, true);
assert.sameValue(iterResult.value, undefined);
