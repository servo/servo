// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.use
description: Throws a TypeError if this is not an Object
info: |
  DisposableStack.prototype.use ( value )

  1. Let disposableStack be the this value.
  2. Perform ? RequireInternalSlot(disposableStack, [[DisposableState]]).
  ...

  RequireInternalSlot ( O, internalSlot )

  1. If O is not an Object, throw a TypeError exception.
  ...

features: [explicit-resource-management]
---*/

assert.sameValue(typeof DisposableStack.prototype.use, 'function');

var use = DisposableStack.prototype.use;

assert.throws(TypeError, function() {
  use.call(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  use.call(null);
}, 'null');

assert.throws(TypeError, function() {
  use.call(true);
}, 'true');

assert.throws(TypeError, function() {
  use.call(false);
}, 'false');

assert.throws(TypeError, function() {
  use.call(1);
}, 'number');

assert.throws(TypeError, function() {
  use.call('object');
}, 'string');

var s = Symbol();
assert.throws(TypeError, function() {
  use.call(s);
}, 'symbol');
