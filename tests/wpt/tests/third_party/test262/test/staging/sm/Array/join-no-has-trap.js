// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/
// Test that Array.prototype.join doesn't call the [[HasProperty]] internal
// method of objects.

var log = [];
var array = [];
var proxy = new Proxy(array, new Proxy({}, {
    get(t, trap, r) {
      return (t, pk, ...more) => {
        log.push(`${trap}:${String(pk)}`);
        return Reflect[trap](t, pk, ...more);
      };
    }
}));

var result;

result = Array.prototype.join.call(proxy);
assert.compareArray(log, [ "get:length" ]);
assert.sameValue(result, "");

log.length = 0;
array.push(1);

result = Array.prototype.join.call(proxy);
assert.compareArray(log, [ "get:length", "get:0" ]);
assert.sameValue(result, "1");

log.length = 0;
array.push(2);

result = Array.prototype.join.call(proxy);
assert.compareArray(log, [ "get:length", "get:0", "get:1" ]);
assert.sameValue(result, "1,2");

