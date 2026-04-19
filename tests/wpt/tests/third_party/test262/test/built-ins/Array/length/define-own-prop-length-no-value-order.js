// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraysetlength
description: >
  Ordinary descriptor validation if [[Value]] is absent.
info: |
  ArraySetLength ( A, Desc )

  1. If Desc.[[Value]] is absent, then
    a. Return OrdinaryDefineOwnProperty(A, "length", Desc).

  OrdinaryDefineOwnProperty ( O, P, Desc )

  [...]
  3. Return ValidateAndApplyPropertyDescriptor(O, P, extensible, Desc, current).

  ValidateAndApplyPropertyDescriptor ( O, P, extensible, Desc, current )

  [...]
  4. If current.[[Configurable]] is false, then
    a. If Desc.[[Configurable]] is present and its value is true, return false.
    b. If Desc.[[Enumerable]] is present and
      ! SameValue(Desc.[[Enumerable]], current.[[Enumerable]]) is false, return false.
  [...]
  6. Else if ! SameValue(! IsDataDescriptor(current), ! IsDataDescriptor(Desc)) is false, then
    a. If current.[[Configurable]] is false, return false.
  [...]
  7. Else if IsDataDescriptor(current) and IsDataDescriptor(Desc) are both true, then
    a. If current.[[Configurable]] is false and current.[[Writable]] is false, then
      i. If Desc.[[Writable]] is present and Desc.[[Writable]] is true, return false.
features: [Reflect]
---*/

assert.throws(TypeError, function() {
  Object.defineProperty([], "length", {configurable: true});
}, 'Object.defineProperty([], "length", {configurable: true}) throws a TypeError exception');

assert(
  !Reflect.defineProperty([], "length", {enumerable: true}),
  'The value of !Reflect.defineProperty([], "length", {enumerable: true}) is expected to be true'
);

assert.throws(TypeError, function() {
  Object.defineProperty([], "length", {
    get: function() {
      throw new Test262Error("[[Get]] shouldn't be called");
    },
  });
}, 'Object.defineProperty([], "length", {get: function() {throw new Test262Error("[[Get]] shouldn"t be called");},}) throws a TypeError exception');

assert(
  !Reflect.defineProperty([], "length", {set: function(_value) {}}),
  'The value of !Reflect.defineProperty([], "length", {set: function(_value) {}}) is expected to be true'
);

var array = [];
Object.defineProperty(array, "length", {writable: false});
assert.throws(TypeError, function() {
  Object.defineProperty(array, "length", {writable: true});
}, 'Object.defineProperty(array, "length", {writable: true}) throws a TypeError exception');
