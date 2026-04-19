// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Explicit iterator closing in response to error
esid: sec-promise.all
info: |
    11. Let result be PerformPromiseAll(iteratorRecord, C, promiseCapability).
    12. If result is an abrupt completion,
        a. If iteratorRecord.[[done]] is false, let result be
           IteratorClose(iterator, result).
        b. IfAbruptRejectPromise(result, promiseCapability).

    [...]

    25.4.4.1.1 Runtime Semantics: PerformPromiseAll

    [...]
    6. Repeat
        [...]
        i. Let nextPromise be Invoke(constructor, "resolve", «nextValue»).
        j. ReturnIfAbrupt(nextPromise ).
features: [Symbol.iterator]
---*/

var iterDoneSpy = {};
var callCount = 0;
iterDoneSpy[Symbol.iterator] = function() {
  return {
    next: function() {
      return {
        value: null,
        done: false
      };
    },
    return: function() {
      callCount += 1;
    }
  };
};

Promise.resolve = function() {
  throw new Test262Error();
};

Promise.all(iterDoneSpy);

assert.sameValue(callCount, 1);
