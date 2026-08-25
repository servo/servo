// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-finalization-registry.prototype.unregister
description: Throws a TypeError if this does not have a [[Cells]] internal slot
info: |
  FinalizationRegistry.prototype.unregister ( unregisterToken )

  1. Let finalizationRegistry be the this value.
  2. If Type(finalizationRegistry) is not Object, throw a TypeError exception.
  3. If finalizationRegistry does not have a [[Cells]] internal slot, throw a TypeError exception.
  4. If Type(unregisterToken) is not Object, throw a TypeError exception.
  ...
features: [WeakSet, WeakMap, FinalizationRegistry, WeakRef]
---*/

assert.sameValue(typeof FinalizationRegistry.prototype.unregister, 'function');

var unregister = FinalizationRegistry.prototype.unregister;
var token = {};

assert.throws(TypeError, function() {
  unregister.call({ ['[[Cells]]']: {} }, token);
}, 'Ordinary object without [[Cells]]');

assert.throws(TypeError, function() {
  unregister.call(WeakRef.prototype, token);
}, 'WeakRef.prototype does not have a [[Cells]] internal slot');

assert.throws(TypeError, function() {
  unregister.call(WeakRef, token);
}, 'WeakRef does not have a [[Cells]] internal slot');

var wr = new WeakRef({});
assert.throws(TypeError, function() {
  unregister.call(wr, token);
}, 'WeakRef instance');

var wm = new WeakMap();
assert.throws(TypeError, function() {
  unregister.call(wm, token);
}, 'WeakMap instance');

var ws = new WeakSet();
assert.throws(TypeError, function() {
  unregister.call(ws, token);
}, 'WeakSet instance');
