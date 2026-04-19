// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.adopt
description: Throws a TypeError if this is not an Object
info: |
  DisposableStack.prototype.adopt ( )

  1. Let disposableStack be the this value.
  2. Perform ? RequireInternalSlot(disposableStack, [[DisposableState]]).
  ...

  RequireInternalSlot ( O, internalSlot )

  1. If O is not an Object, throw a TypeError exception.
  ...

features: [explicit-resource-management]
---*/

assert.sameValue(typeof DisposableStack.prototype.adopt, 'function');

var adopt = DisposableStack.prototype.adopt;

assert.throws(TypeError, function() {
  adopt.call(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  adopt.call(null);
}, 'null');

assert.throws(TypeError, function() {
  adopt.call(true);
}, 'true');

assert.throws(TypeError, function() {
  adopt.call(false);
}, 'false');

assert.throws(TypeError, function() {
  adopt.call(1);
}, 'number');

assert.throws(TypeError, function() {
  adopt.call('object');
}, 'string');

var s = Symbol();
assert.throws(TypeError, function() {
  adopt.call(s);
}, 'symbol');
