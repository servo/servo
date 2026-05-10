// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.foreach
description: >
  Repeats for each non-empty record, in original key insertion order.
info: |
  Map.prototype.forEach ( callbackfn [ , thisArg ] )

  ...
  5. If thisArg was supplied, let T be thisArg; else let T be undefined.
  6. Let entries be the List that is the value of M’s [[MapData]] internal slot.
  7. Repeat for each Record {[[key]], [[value]]} e that is an element of
  entries, in original key insertion order
    a. If e.[[key]] is not empty, then
      i. Let funcResult be Call(callbackfn, T, «e.[[value]], e.[[key]], M»).
  ...
---*/

var map = new Map([
  ['foo', 'valid foo'],
  ['bar', false],
  ['baz', 'valid baz']
]);
map.set(0, false);
map.set(1, false);
map.set(2, 'valid 2');
map.delete(1);
map.delete('bar');

// Not setting a new key, just changing the value
map.set(0, 'valid 0');

var results = [];
var callback = function(value) {
  results.push(value);
};

map.forEach(callback);

assert.sameValue(results[0], 'valid foo');
assert.sameValue(results[1], 'valid baz');
assert.sameValue(results[2], 'valid 0');
assert.sameValue(results[3], 'valid 2');
assert.sameValue(results.length, 4);

map.clear();
results = [];

map.forEach(callback);
assert.sameValue(results.length, 0);
