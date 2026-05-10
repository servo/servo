// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraysetlength
description: >
  [[Value]] is checked for overflow before descriptor validation.
info: |
  ArraySetLength ( A, Desc )

  [...]
  3. Let newLen be ? ToUint32(Desc.[[Value]]).
  4. Let numberLen be ? ToNumber(Desc.[[Value]]).
  5. If newLen ≠ numberLen, throw a RangeError exception.
---*/

assert.throws(RangeError, function() {
  Object.defineProperty([], "length", {value: -1, configurable: true});
}, 'Object.defineProperty([], "length", {value: -1, configurable: true}) throws a RangeError exception');

assert.throws(RangeError, function() {
  Object.defineProperty([], "length", {value: NaN, enumerable: true});
}, 'Object.defineProperty([], "length", {value: NaN, enumerable: true}) throws a RangeError exception');

var array = [];
Object.defineProperty(array, "length", {writable: false});
assert.throws(RangeError, function() {
  Object.defineProperty(array, "length", {value: Number.MAX_SAFE_INTEGER, writable: true});
}, 'Object.defineProperty(array, "length", {value: Number.MAX_SAFE_INTEGER, writable: true}) throws a RangeError exception');
