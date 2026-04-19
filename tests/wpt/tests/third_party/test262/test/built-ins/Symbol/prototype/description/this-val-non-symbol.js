// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-symbol.prototype.description
description: >
    Behavior when "this" value is an object without a [[SymbolData]] internal
    slot.
info: |
    1. Let s be the this value.
    2. Let sym be ? thisSymbolValue(s).
    3. Return sym.[[Description]].
features: [Symbol.prototype.description]
---*/

const getter = Object.getOwnPropertyDescriptor(
  Symbol.prototype, 'description'
).get;

assert.throws(TypeError, function() {
  getter.call(null);
}, 'getter.call(null) throws TypeError');

assert.throws(TypeError, function() {
  getter.call(123);
}, 'getter.call(123) throws TypeError');

assert.throws(TypeError, function() {
  getter.call('test');
}, 'getter.call("test") throws TypeError');

assert.throws(TypeError, function() {
  getter.call(true);
}, 'getter.call(true) throws TypeError');

assert.throws(TypeError, function() {
  getter.call(undefined);
}, 'getter.call(undefined) throws TypeError');

assert.throws(TypeError, function() {
  getter.call(new Proxy({}, {}));
}, 'getter.call(new Proxy({}, {})) throws TypeError');

assert.throws(TypeError, function() {
  getter.call({});
}, 'getter.call({}) throws TypeError');
