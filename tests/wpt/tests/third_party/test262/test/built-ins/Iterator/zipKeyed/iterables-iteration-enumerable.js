// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  Non-enumerable properties are skipped in "iterables" iteration.
info: |
  Iterator.zipKeyed ( iterables [ , options ] )
    ...
    10. Let allKeys be ? iterables.[[OwnPropertyKeys]]().
    11. Let keys be a new empty List.
    12. For each element key of allKeys, do
      a. Let desc be Completion(iterables.[[GetOwnProperty]](key)).
      ...
      c. If desc is not undefined and desc.[[Enumerable]] is true, then
        ...
includes: [compareArray.js]
features: [joint-iteration]
---*/

var log = [];

var iterables = Object.create(null, {
  a: {
    enumerable: false,
    get() {
      throw new Test262Error("unexpected get a");
    }
  },
  b: {
    enumerable: true,
    get() {
      log.push("get b");

      // Change enumerable of property "c".
      Object.defineProperty(iterables, "c", {
        enumerable: false,
      });

      return ['value for b'];
    }
  },
  c: {
    enumerable: true,
    configurable: true,
    get() {
      throw new Test262Error("unexpected get c");
    }
  },
  d: {
    enumerable: true,
    get() {
      log.push("get d");

      // Change enumerable of property "e".
      Object.defineProperty(iterables, "e", {
        enumerable: true,
      });

      return ['value for d'];
    }
  },
  e: {
    enumerable: false,
    configurable: true,
    get() {
      log.push("get e");
      return ['value for e'];
    }
  },
});

var result = Array.from(Iterator.zipKeyed(iterables));

assert.compareArray(log, [
  "get b",
  "get d",
  "get e",
]);

assert.sameValue(result.length, 1);
assert.compareArray(Object.keys(result[0]), ["b", "d", "e"]);
assert.compareArray(Object.values(result[0]), ["value for b", "value for d", "value for e"]);
