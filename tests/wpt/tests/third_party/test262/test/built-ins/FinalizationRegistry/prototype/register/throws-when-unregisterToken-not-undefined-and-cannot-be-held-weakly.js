// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-finalization-registry.prototype.register
description: >
  Throws a TypeError if unregisterToken is not undefined and cannot be held
  weakly
info: |
  FinalizationRegistry.prototype.register ( _target_ , _heldValue_ [, _unregisterToken_ ] )
  5. If CanBeHeldWeakly(_unregisterToken_) is *false*, then
    a. If _unregisterToken_ is not *undefined*, throw a *TypeError* exception.
features: [FinalizationRegistry]
---*/

assert.sameValue(typeof FinalizationRegistry.prototype.register, 'function');

var finalizationRegistry = new FinalizationRegistry(function() {});
var target = {};

assert.throws(TypeError, function() {
  finalizationRegistry.register(target, undefined, null);
}, 'null');

assert.throws(TypeError, function() {
  finalizationRegistry.register(target, undefined, true);
}, 'true');

assert.throws(TypeError, function() {
  finalizationRegistry.register(target, undefined, false);
}, 'false');

assert.throws(TypeError, function() {
  finalizationRegistry.register(target, undefined, 1);
}, 'number');

assert.throws(TypeError, function() {
  finalizationRegistry.register(target, undefined, 'object');
}, 'string');

var s = Symbol.for('registered symbol');
assert.throws(TypeError, function() {
  finalizationRegistry.register(target, undefined, s);
}, 'registered symbol');
