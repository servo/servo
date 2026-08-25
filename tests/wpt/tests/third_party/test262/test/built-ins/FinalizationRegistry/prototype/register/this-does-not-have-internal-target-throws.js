// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-finalization-registry.prototype.register
description: Throws a TypeError if this does not have a [[Cells]] internal slot
info: |
  FinalizationRegistry.prototype.register ( target , holdings [, unregisterToken ] )

  1. Let finalizationRegistry be the this value.
  2. If Type(finalizationRegistry) is not Object, throw a TypeError exception.
  3. If Type(target) is not Object, throw a TypeError exception.
  4. If finalizationRegistry does not have a [[Cells]] internal slot, throw a TypeError exception.
  ...
features: [WeakSet, WeakMap, FinalizationRegistry, WeakRef]
---*/

assert.sameValue(typeof FinalizationRegistry.prototype.register, 'function');

var register = FinalizationRegistry.prototype.register;
var target = {};

assert.throws(TypeError, function() {
  register.call({ ['[[Cells]]']: {} }, target);
}, 'Ordinary object without [[Cells]]');

assert.throws(TypeError, function() {
  register.call(WeakRef.prototype, target);
}, 'WeakRef.prototype does not have a [[Cells]] internal slot');

assert.throws(TypeError, function() {
  register.call(WeakRef, target);
}, 'WeakRef does not have a [[Cells]] internal slot');

var wr = new WeakRef({});
assert.throws(TypeError, function() {
  register.call(wr, target);
}, 'WeakRef instance');

var wm = new WeakMap();
assert.throws(TypeError, function() {
  register.call(wm, target);
}, 'WeakMap instance');

var ws = new WeakSet();
assert.throws(TypeError, function() {
  register.call(ws, target);
}, 'WeakSet instance');
