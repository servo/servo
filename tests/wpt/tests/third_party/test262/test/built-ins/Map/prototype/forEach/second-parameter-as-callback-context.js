// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.foreach
description: >
  If a thisArg parameter is provided, it will be used as the this value for each
  invocation of callbackfn.
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

var expectedThis = {};
var _this = [];

var map = new Map();
map.set(0, 0);
map.set(1, 1);
map.set(2, 2);

var callback = function() {
  _this.push(this);
};

map.forEach(callback, expectedThis);

assert.sameValue(_this[0], expectedThis);
assert.sameValue(_this[1], expectedThis);
assert.sameValue(_this[2], expectedThis);
