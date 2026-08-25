// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-finalization-registry.prototype.register
description: Throws a TypeError if target cannot be held weakly
info: |
  FinalizationRegistry.prototype.register ( _target_ , _heldValue_ [, _unregisterToken_ ] )
  3. If CanBeHeldWeakly(_target_) is *false*, throw a *TypeError* exception.
features: [FinalizationRegistry]
---*/

assert.sameValue(typeof FinalizationRegistry.prototype.register, 'function');

var finalizationRegistry = new FinalizationRegistry(function() {});

assert.throws(TypeError, function() {
  finalizationRegistry.register(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  finalizationRegistry.register(null);
}, 'null');

assert.throws(TypeError, function() {
  finalizationRegistry.register(true);
}, 'true');

assert.throws(TypeError, function() {
  finalizationRegistry.register(false);
}, 'false');

assert.throws(TypeError, function() {
  finalizationRegistry.register(1);
}, 'number');

assert.throws(TypeError, function() {
  finalizationRegistry.register('object');
}, 'string');

var s = Symbol.for('registered symbol');
assert.throws(TypeError, function() {
  finalizationRegistry.register(s);
}, 'registered symbol');
