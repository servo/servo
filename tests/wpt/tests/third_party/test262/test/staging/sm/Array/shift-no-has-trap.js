// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/
// Test that Array.prototype.shift doesn't call the [[HasProperty]] internal
// method of objects when retrieving the element at index 0.

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

result = Array.prototype.shift.call(proxy);
assert.compareArray(log, [
  "get:length",
  "set:length", "getOwnPropertyDescriptor:length", "defineProperty:length"
]);
assert.sameValue(result, undefined);

log.length = 0;
array.push(1);

result = Array.prototype.shift.call(proxy);
assert.compareArray(log, [
  "get:length",
  "get:0",
  "deleteProperty:0",
  "set:length", "getOwnPropertyDescriptor:length", "defineProperty:length"
]);
assert.sameValue(result, 1);

log.length = 0;
array.push(2, 3);

result = Array.prototype.shift.call(proxy);
assert.compareArray(log, [
  "get:length",
  "get:0",
  "has:1", "get:1", "set:0", "getOwnPropertyDescriptor:0", "defineProperty:0",
  "deleteProperty:1",
  "set:length", "getOwnPropertyDescriptor:length", "defineProperty:length"
]);
assert.sameValue(result, 2);

log.length = 0;
array.push(4, 5);

result = Array.prototype.shift.call(proxy);
assert.compareArray(log, [
  "get:length",
  "get:0",
  "has:1", "get:1", "set:0", "getOwnPropertyDescriptor:0", "defineProperty:0",
  "has:2", "get:2", "set:1", "getOwnPropertyDescriptor:1", "defineProperty:1",
  "deleteProperty:2",
  "set:length", "getOwnPropertyDescriptor:length", "defineProperty:length"
]);
assert.sameValue(result, 3);

