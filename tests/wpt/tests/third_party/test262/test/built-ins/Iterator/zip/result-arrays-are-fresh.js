// Copyright (C) 2026 Test262 Contributors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zip
description: >
  Each call to next() returns a fresh array, not the same object reused.
info: |
  IteratorZip ( iters, mode, padding, finishResults )
    ...
    3. Let closure be a new Abstract Closure with no parameters that captures
       iters, iterCount, openIters, mode, padding, and finishResults, and
       performs the following steps when called:
      ...
      b. Repeat,
        ...
        iv. Let results be finishResults(results).
        v. Let completion be Completion(Yield(results)).
    ...

  Iterator.zip ( iterables [ , options ] )
    ...
    15. Let finishResults be a new Abstract Closure with parameters (results)
        that captures nothing and performs the following steps when called:
      a. Return CreateArrayFromList(results).
    ...
includes: [compareArray.js]
features: [joint-iteration]
---*/

var it = Iterator.zip([[1, 2, 3], [4, 5, 6]]);

var first = it.next();
assert.sameValue(first.done, false);

var second = it.next();
assert.sameValue(second.done, false);

var third = it.next();
assert.sameValue(third.done, false);

assert.notSameValue(first.value, second.value);
assert.notSameValue(second.value, third.value);
assert.notSameValue(first.value, third.value);

assert.compareArray(first.value, [1, 4]);
assert.compareArray(second.value, [2, 5]);
assert.compareArray(third.value, [3, 6]);
