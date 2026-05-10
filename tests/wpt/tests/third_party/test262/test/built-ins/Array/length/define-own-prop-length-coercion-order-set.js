// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: André Bargull
esid: sec-arraysetlength
description: >
  [[Value]] is coerced to number before current descriptor's [[Writable]] check.
info: |
  ArraySetLength ( A, Desc )

  [...]
  3. Let newLen be ? ToUint32(Desc.[[Value]]).
  4. Let numberLen be ? ToNumber(Desc.[[Value]]).
  [...]
  7. Let oldLenDesc be OrdinaryGetOwnProperty(A, "length").
  [...]
  12. If oldLenDesc.[[Writable]] is false, return false.
features: [Symbol, Symbol.toPrimitive, Reflect, Reflect.set]
includes: [compareArray.js]
---*/

var array = [1, 2, 3];
var hints = [];
var length = {};
length[Symbol.toPrimitive] = function(hint) {
  hints.push(hint);
  Object.defineProperty(array, "length", {writable: false});
  return 0;
};

assert.throws(TypeError, function() {
  "use strict";
  array.length = length;
}, '`"use strict"; array.length = length` throws a TypeError exception');
assert.compareArray(hints, ["number", "number"], 'The value of hints is expected to be ["number", "number"]');


array = [1, 2, 3];
hints = [];

assert(
  !Reflect.set(array, "length", length),
  'The value of !Reflect.set(array, "length", length) is expected to be true'
);
assert.compareArray(hints, ["number", "number"], 'The value of hints is expected to be ["number", "number"]');
