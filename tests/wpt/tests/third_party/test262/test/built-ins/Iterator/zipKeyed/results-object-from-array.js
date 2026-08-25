// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  Calling Iterator.zipKeyed with an array object.
info: |
  Iterator.zipKeyed ( iterables [ , options ] )
    ...
    15. Let finishResults be a new Abstract Closure with parameters (results) that captures keys and iterCount and performs the following steps when called:
      a. Let obj be OrdinaryObjectCreate(null).
      b. For each integer i such that 0 ≤ i < iterCount, in ascending order, do
        i. Perform ! CreateDataPropertyOrThrow(obj, keys[i], results[i]).
      c. Return obj.
    ...
features: [joint-iteration]
---*/

var iterables = [
  [1, 2, 3],
  [4, 5, 6],
];

var it = Iterator.zipKeyed(iterables);

for (var i = 0; i < iterables[0].length; ++i) {
  var results = it.next().value;

  assert.sameValue(
    Object.getPrototypeOf(results),
    null,
    "results prototype is null"
  );

  assert.sameValue(
    Reflect.ownKeys(results).length,
    iterables.length,
    "results has correct number of properties"
  );

  for (var j = 0; j < iterables.length; ++j) {
    assert.sameValue(
      results[j],
      iterables[j][i],
      "results property value has the correct value"
    );
  }
}

assert.sameValue(it.next().done, true, "iterator is exhausted");
