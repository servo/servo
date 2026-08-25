// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.from
description: Throw a TypeError exception is mapfn is not callable
info: |
  22.2.2.1 %TypedArray%.from ( source [ , mapfn [ , thisArg ] ] )

  ...
  3. If mapfn was supplied and mapfn is not undefined, then
    a. If IsCallable(mapfn) is false, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [Symbol, Symbol.iterator, TypedArray]
---*/

var getIterator = 0;
var arrayLike = {};
Object.defineProperty(arrayLike, Symbol.iterator, {
  get: function() {
    getIterator++;
  }
});

assert.throws(TypeError, function() {
  TypedArray.from(arrayLike, null);
}, "mapfn is null");

assert.throws(TypeError, function() {
  TypedArray.from(arrayLike, 42);
}, "mapfn is a number");

assert.throws(TypeError, function() {
  TypedArray.from(arrayLike, "");
}, "mapfn is a string");

assert.throws(TypeError, function() {
  TypedArray.from(arrayLike, {});
}, "mapfn is an ordinary object");

assert.throws(TypeError, function() {
  TypedArray.from(arrayLike, []);
}, "mapfn is an array");

assert.throws(TypeError, function() {
  TypedArray.from(arrayLike, true);
}, "mapfn is a boolean");

var s = Symbol("1");
assert.throws(TypeError, function() {
  TypedArray.from(arrayLike, s);
}, "mapfn is a symbol");

assert.sameValue(
  getIterator, 0,
  "IsCallable(mapfn) check occurs before getting source[@@iterator]"
);
