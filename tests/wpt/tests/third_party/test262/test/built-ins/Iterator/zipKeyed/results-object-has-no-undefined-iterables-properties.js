// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  Undefined properties from the "iterables" object are not present in the results object
info: |
  Iterator.zipKeyed ( iterables [ , options ] )
    ...
    12. For each element key of allKeys, do
      a. Let desc be Completion(iterables.[[GetOwnProperty]](key)).
      b. IfAbruptCloseIterators(desc, iters).
      c. If desc is not undefined and desc.[[Enumerable]] is true, then
        ...
features: [joint-iteration]
---*/

var iterables = {
  a: ["A"],
  b: undefined,
  c: ["C"],
};

var it = Iterator.zipKeyed(iterables);

var results = it.next().value;

assert.sameValue("a" in results, true, "property 'a' is present");
assert.sameValue("b" in results, false, "property 'b' is not present");
assert.sameValue("c" in results, true, "property 'c' is present");

assert.sameValue(it.next().done, true, "iterator is exhausted");
