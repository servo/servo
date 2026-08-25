// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 25.4.4.5
description: >
  Throws a TypeError if `this` is not an Object.
info: |
  Promise.resolve ( x )

  1. Let C be the this value.
  2. If Type(C) is not Object, throw a TypeError exception.
  ...
features: [Symbol]
---*/

var promise = new Promise(function() {});

promise.constructor = undefined;
assert.throws(TypeError, function() {
  Promise.resolve.call(undefined, promise);
}, "`this` value is undefined");

promise.constructor = null;
assert.throws(TypeError, function() {
  Promise.resolve.call(null, promise);
}, "`this` value is null");

promise.constructor = true;
assert.throws(TypeError, function() {
  Promise.resolve.call(true, promise);
}, "`this` value is a Boolean");

promise.constructor = 1;
assert.throws(TypeError, function() {
  Promise.resolve.call(1, promise);
}, "`this` value is a Number");

promise.constructor = "";
assert.throws(TypeError, function() {
  Promise.resolve.call("", promise);
}, "`this` value is a String");

var symbol = Symbol();
promise.constructor = symbol;
assert.throws(TypeError, function() {
  Promise.resolve.call(symbol, promise);
}, "`this` value is a Symbol");
