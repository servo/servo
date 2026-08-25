// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-finalization-registry.prototype.unregister
description: Throws a TypeError if unregisterToken cannot be held weakly
info: |
  FinalizationRegistry.prototype.unregister ( _unregisterToken_ )
  3. If CanBeHeldWeakly(_unregisterToken_) is *false*, throw a *TypeError* exception.
features: [FinalizationRegistry]
---*/

assert.sameValue(typeof FinalizationRegistry.prototype.unregister, 'function');

var finalizationRegistry = new FinalizationRegistry(function() {});

assert.throws(TypeError, function() {
  finalizationRegistry.unregister(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  finalizationRegistry.unregister(null);
}, 'null');

assert.throws(TypeError, function() {
  finalizationRegistry.unregister(true);
}, 'true');

assert.throws(TypeError, function() {
  finalizationRegistry.unregister(false);
}, 'false');

assert.throws(TypeError, function() {
  finalizationRegistry.unregister(1);
}, 'number');

assert.throws(TypeError, function() {
  finalizationRegistry.unregister('object');
}, 'string');

var s = Symbol.for('registered symbol');
assert.throws(TypeError, function() {
  finalizationRegistry.unregister(s);
}, 'registered symbol');
