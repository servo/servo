// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.foreach
description: >
  Verify the parameters order on the given callback.
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

var map = new Map();
map.set('foo', 42);
map.set('bar', 'baz');

var results = [];

var callback = function(value, key, thisArg) {
  results.push({
    value: value,
    key: key,
    thisArg: thisArg
  });
};

map.forEach(callback);

assert.sameValue(results[0].value, 42);
assert.sameValue(results[0].key, 'foo');
assert.sameValue(results[0].thisArg, map);

assert.sameValue(results[1].value, 'baz');
assert.sameValue(results[1].key, 'bar');
assert.sameValue(results[1].thisArg, map);

assert.sameValue(results.length, 2);
