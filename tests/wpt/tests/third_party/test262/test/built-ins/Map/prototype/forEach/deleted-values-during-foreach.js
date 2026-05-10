// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.foreach
description: >
  Map state with deleted values during forEach.
info: |
  Map.prototype.forEach ( callbackfn [ , thisArg ] )

  ...
  5. If thisArg was supplied, let T be thisArg; else let T be undefined.
  6. Let entries be the List that is the value of M’s [[MapData]] internal slot.
  7. Repeat for each Record {[[key]], [[value]]} e that is an element of
  entries, in original key insertion order
    a. If e.[[key]] is not empty, then
      i. Let funcResult be Call(callbackfn, T, «e.[[value]], e.[[key]], M»).
      ii. ReturnIfAbrupt(funcResult).
  ...
---*/

var map = new Map();
map.set('foo', 0);
map.set('bar', 1);

var count = 0;
var results = [];

map.forEach(function(value, key) {
  if (count === 0) {
    map.delete('bar');
  }
  results.push({
    value: value,
    key: key
  });
  count++;
});

assert.sameValue(results.length, 1);
assert.sameValue(results[0].key, 'foo');
assert.sameValue(results[0].value, 0);
