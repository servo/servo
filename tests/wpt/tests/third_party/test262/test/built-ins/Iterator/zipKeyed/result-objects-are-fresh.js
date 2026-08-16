// Copyright (C) 2026 Test262 Contributors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  Each call to next() returns a fresh result object, not the same object reused.
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

  Iterator.zipKeyed ( iterables [ , options ] )
    ...
    15. Let finishResults be a new Abstract Closure with parameters (results) that captures keys
        and iterCount and performs the following steps when called:
      a. Let obj be OrdinaryObjectCreate(null).
      b. For each integer i such that 0 ≤ i < iterCount, in ascending order, do
        i. Perform ! CreateDataPropertyOrThrow(obj, keys[i], results[i]).
      c. Return obj.
    ...
features: [joint-iteration]
---*/

var it = Iterator.zipKeyed({ a: [1, 2, 3], b: [4, 5, 6] });

var first = it.next();
assert.sameValue(first.done, false);

var second = it.next();
assert.sameValue(second.done, false);

var third = it.next();
assert.sameValue(third.done, false);

assert.notSameValue(first.value, second.value);
assert.notSameValue(second.value, third.value);
assert.notSameValue(first.value, third.value);

assert.sameValue(first.value.a, 1);
assert.sameValue(first.value.b, 4);
assert.sameValue(second.value.a, 2);
assert.sameValue(second.value.b, 5);
assert.sameValue(third.value.a, 3);
assert.sameValue(third.value.b, 6);
